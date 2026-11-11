import 'dart:async';

import 'package:flutter/material.dart';
import 'package:matrix_client/src/rust/api/matrix.dart';
import 'package:matrix_client/theme.dart';
import 'package:matrix_client/widgets/member_drawer.dart';
import 'package:matrix_client/widgets/rail_nav.dart';
import 'package:matrix_client/widgets/room_list.dart';
import 'package:matrix_client/widgets/room_view.dart';

class HomeScreen extends StatefulWidget {
  const HomeScreen({super.key, required this.client, required this.onLogout});

  final MatrixClient client;
  final VoidCallback onLogout;

  @override
  State<HomeScreen> createState() => _HomeScreenState();
}

class _HomeScreenState extends State<HomeScreen> {
  final _messageController = TextEditingController();

  String? _error;
  bool _busy = false;

  List<RoomSummary> _rooms = [];
  RoomSummary? _selectedRoom;
  RoomTimeline? _timeline;
  List<TimelineMessage> _messages = [];
  StreamSubscription<List<TimelineMessage>>? _timelineSubscription;

  RailTab? _activeTab = RailTab.home;
  String? _userId;
  String? _displayName;

  List<SpaceSummary> _spaces = [];
  SpaceSummary? _activeSpace;
  List<RoomSummary> _spaceRooms = [];
  StreamSubscription<List<RoomSummary>>? _roomsSubscription;
  StreamSubscription<List<SpaceSummary>>? _spacesSubscription;

  List<RoomSummary> _directRooms = [];
  StreamSubscription<List<RoomSummary>>? _directRoomsSubscription;

  bool _membersOpen = false;
  bool _membersBusy = false;
  List<RoomMemberSummary> _members = [];

  @override
  void initState() {
    super.initState();
    _startSync();
    _loadSession();
    // Real push updates for both lists (matrix_sdk::Client::rooms_stream
    // / SpaceService::subscribe_to_top_level_joined_spaces, re-flattened
    // to plain snapshots) — no fixed post-sync delay needed, the first
    // emission just arrives whenever the store actually has data.
    _roomsSubscription = roomsStream(client: widget.client).listen(
      (roomList) {
        if (mounted) setState(() => _rooms = roomList);
      },
      onError: (e) {
        if (mounted) setState(() => _error = e.toString());
      },
    );
    _spacesSubscription = spacesStream(client: widget.client).listen(
      (spaceList) {
        if (mounted) setState(() => _spaces = spaceList);
      },
      onError: (e) {
        if (mounted) setState(() => _error = e.toString());
      },
    );
    _directRoomsSubscription = directRoomsStream(client: widget.client).listen(
      (roomList) {
        if (mounted) setState(() => _directRooms = roomList);
      },
      onError: (e) {
        if (mounted) setState(() => _error = e.toString());
      },
    );
  }

  Future<void> _loadSession() async {
    final session = await clientSession(client: widget.client);
    if (mounted) setState(() => _userId = session?.userId);
    try {
      final name = await accountDisplayName(client: widget.client);
      if (mounted) setState(() => _displayName = name);
    } catch (_) {
      // Not critical — the settings panel just falls back to the user
      // ID if this fails, same as every other display-name fallback in
      // the app.
    }
  }

  @override
  void dispose() {
    _timelineSubscription?.cancel();
    _stopCurrentTimeline();
    _roomsSubscription?.cancel();
    _spacesSubscription?.cancel();
    _directRoomsSubscription?.cancel();
    _messageController.dispose();
    super.dispose();
  }

  Future<void> _startSync() async {
    setState(() => _busy = true);
    try {
      await startSync(client: widget.client);
    } catch (e) {
      setState(() => _error = e.toString());
    } finally {
      if (mounted) setState(() => _busy = false);
    }
  }

  // Manual-refresh button on the Home room list — a one-shot re-fetch on
  // top of the live `_roomsSubscription`, kept mainly as a user-visible
  // "nudge it" affordance; the list updates on its own regardless.
  Future<void> _refreshRooms() async {
    try {
      final roomList = await rooms(client: widget.client);
      setState(() => _rooms = roomList);
    } catch (e) {
      setState(() => _error = e.toString());
    }
  }

  // Manual-refresh button on the Direct Messages room list — same
  // one-shot re-fetch pattern as `_refreshRooms`, on top of the live
  // `_directRoomsSubscription`.
  Future<void> _refreshDirectRooms() async {
    try {
      final roomList = await directRooms(client: widget.client);
      setState(() => _directRooms = roomList);
    } catch (e) {
      setState(() => _error = e.toString());
    }
  }

  void _selectTab(RailTab tab) {
    setState(() {
      _activeTab = tab;
      _activeSpace = null;
    });
  }

  void _toggleMembers() {
    if (_membersOpen) {
      setState(() => _membersOpen = false);
      return;
    }
    setState(() => _membersOpen = true);
    _loadMembers();
  }

  Future<void> _loadMembers() async {
    final room = _selectedRoom;
    if (room == null) return;
    setState(() => _membersBusy = true);
    try {
      final memberList = await roomMembers(
        client: widget.client,
        roomId: room.roomId,
      );
      if (mounted) setState(() => _members = memberList);
    } catch (e) {
      if (mounted) setState(() => _error = e.toString());
    } finally {
      if (mounted) setState(() => _membersBusy = false);
    }
  }

  Future<void> _selectSpace(SpaceSummary space) async {
    _timelineSubscription?.cancel();
    _stopCurrentTimeline();
    setState(() {
      _activeTab = null;
      _activeSpace = space;
      _selectedRoom = null;
      _timeline = null;
      _messages = [];
      _membersOpen = false;
      _members = [];
    });
    await _refreshSpaceRooms();
  }

  Future<void> _refreshSpaceRooms() async {
    final space = _activeSpace;
    if (space == null) return;
    setState(() => _busy = true);
    try {
      final roomList = await spaceRooms(
        client: widget.client,
        spaceId: space.roomId,
      );
      setState(() => _spaceRooms = roomList);
    } catch (e) {
      setState(() => _error = e.toString());
    } finally {
      if (mounted) setState(() => _busy = false);
    }
  }

  // Stops the Rust-side push loop for whatever `_timeline` currently is,
  // if any, and disposes the underlying native `Timeline` — without the
  // `.dispose()` call, switching away from a quiet room leaked the
  // native `Timeline` (and its background sync task) until Dart's
  // GC/FRB finalizer eventually got around to it, same class of leak as
  // `MatrixClient` on logout (`main.dart`'s `_logout`). `stopTimeline`
  // alone only cancels our own forwarding-stream wrapper, not the
  // underlying object. Cancelling the Dart subscription alone isn't
  // enough either; call this too.
  void _stopCurrentTimeline() {
    final timeline = _timeline;
    if (timeline != null) {
      stopTimeline(timeline: timeline);
      timeline.dispose();
    }
  }

  Future<void> _openRoom(RoomSummary room) async {
    await _timelineSubscription?.cancel();
    _stopCurrentTimeline();
    setState(() {
      _busy = true;
      _error = null;
    });
    try {
      final timeline = await timelineFor(
        client: widget.client,
        roomId: room.roomId,
      );
      setState(() {
        _selectedRoom = room;
        _timeline = timeline;
        _messages = [];
        _members = [];
      });
      // Members panel stays open across a room switch (matches how the
      // design mockup treats it) — just refetch for the newly-opened
      // room instead of closing it.
      if (_membersOpen) unawaited(_loadMembers());
      // Real push updates via matrix-client's `subscribe_messages`
      // (Timeline::subscribe_filter_map's VectorDiff stream, re-flattened
      // to a plain snapshot per update) — the first emission is the
      // current snapshot, every one after that is a live update. No more
      // polling.
      _timelineSubscription = timelineMessagesStream(timeline: timeline)
          .listen(
            (messages) {
              if (mounted) setState(() => _messages = messages);
            },
            onError: (e) {
              if (mounted) setState(() => _error = e.toString());
            },
          );
    } catch (e) {
      setState(() => _error = e.toString());
    } finally {
      if (mounted) setState(() => _busy = false);
    }
  }

  Future<void> _sendMessage() async {
    final timeline = _timeline;
    if (timeline == null || _messageController.text.isEmpty) return;
    setState(() => _busy = true);
    try {
      await sendText(timeline: timeline, body: _messageController.text);
      // No manual refetch needed — the active `_timelineSubscription`
      // picks up the local echo (and its later confirmation) on its own.
      _messageController.clear();
    } catch (e) {
      setState(() => _error = e.toString());
    } finally {
      if (mounted) setState(() => _busy = false);
    }
  }

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      backgroundColor: MatrixColors.bgContainer,
      body: SafeArea(
        child: Column(
          children: [
            if (_error != null)
              Container(
                width: double.infinity,
                color: MatrixColors.criticalContainer,
                padding: const EdgeInsets.all(8),
                child: Text(
                  _error!,
                  style: const TextStyle(color: MatrixColors.criticalMain),
                ),
              ),
            Expanded(
              child: Row(
                children: [
                  RailNav(
                    activeTab: _activeTab,
                    spaces: _spaces,
                    activeSpaceId: _activeSpace?.roomId,
                    userId: _userId,
                    onSelect: _selectTab,
                    onSelectSpace: _selectSpace,
                  ),
                  const VerticalDivider(
                    width: 1,
                    color: MatrixColors.bgContainerLine,
                  ),
                  if (_activeSpace != null) ...[
                    RoomList(
                      title: _activeSpace!.displayName,
                      rooms: _spaceRooms,
                      selectedRoomId: _selectedRoom?.roomId,
                      onSelect: _openRoom,
                      onRefresh: _refreshSpaceRooms,
                      busy: _busy,
                    ),
                    const VerticalDivider(
                      width: 1,
                      color: MatrixColors.bgContainerLine,
                    ),
                    RoomView(
                      room: _selectedRoom,
                      messages: _messages,
                      messageController: _messageController,
                      onSend: _sendMessage,
                      busy: _busy,
                      membersOpen: _membersOpen,
                      onToggleMembers: _toggleMembers,
                    ),
                    if (_membersOpen && _selectedRoom != null) ...[
                      const VerticalDivider(
                        width: 1,
                        color: MatrixColors.bgContainerLine,
                      ),
                      MemberDrawer(members: _members, busy: _membersBusy),
                    ],
                  ] else if (_activeTab == RailTab.home) ...[
                    RoomList(
                      rooms: _rooms,
                      selectedRoomId: _selectedRoom?.roomId,
                      onSelect: _openRoom,
                      onRefresh: _refreshRooms,
                      busy: _busy,
                    ),
                    const VerticalDivider(
                      width: 1,
                      color: MatrixColors.bgContainerLine,
                    ),
                    RoomView(
                      room: _selectedRoom,
                      messages: _messages,
                      messageController: _messageController,
                      onSend: _sendMessage,
                      busy: _busy,
                      membersOpen: _membersOpen,
                      onToggleMembers: _toggleMembers,
                    ),
                    if (_membersOpen && _selectedRoom != null) ...[
                      const VerticalDivider(
                        width: 1,
                        color: MatrixColors.bgContainerLine,
                      ),
                      MemberDrawer(members: _members, busy: _membersBusy),
                    ],
                  ] else if (_activeTab == RailTab.direct) ...[
                    RoomList(
                      title: 'Direct Messages',
                      rooms: _directRooms,
                      selectedRoomId: _selectedRoom?.roomId,
                      onSelect: _openRoom,
                      onRefresh: _refreshDirectRooms,
                      busy: _busy,
                    ),
                    const VerticalDivider(
                      width: 1,
                      color: MatrixColors.bgContainerLine,
                    ),
                    RoomView(
                      room: _selectedRoom,
                      messages: _messages,
                      messageController: _messageController,
                      onSend: _sendMessage,
                      busy: _busy,
                      membersOpen: _membersOpen,
                      onToggleMembers: _toggleMembers,
                    ),
                    if (_membersOpen && _selectedRoom != null) ...[
                      const VerticalDivider(
                        width: 1,
                        color: MatrixColors.bgContainerLine,
                      ),
                      MemberDrawer(members: _members, busy: _membersBusy),
                    ],
                  ] else if (_activeTab == RailTab.settings) ...[
                    Expanded(
                      child: _SettingsPanel(
                        userId: _userId,
                        displayName: _displayName,
                        onLogout: widget.onLogout,
                      ),
                    ),
                  ] else
                    const Expanded(child: _ComingSoon()),
                ],
              ),
            ),
          ],
        ),
      ),
    );
  }
}

// Real profile display (avatar, display name, user ID) plus the logout
// action from the mockup's settings dialog. Everything else the mockup
// has (profile edit, appearance, notifications) isn't backed by any
// matrix-client API yet, so it stays out rather than faking it. Plain
// content, not styled like the mockup's modal — that's still
// deliberately deferred to real UI work.
class _SettingsPanel extends StatelessWidget {
  const _SettingsPanel({
    required this.userId,
    required this.displayName,
    required this.onLogout,
  });

  final String? userId;
  final String? displayName;
  final VoidCallback onLogout;

  @override
  Widget build(BuildContext context) {
    final userId = this.userId;
    final name = displayName ?? userId?.replaceFirst('@', '') ?? '?';
    return ColoredBox(
      color: MatrixColors.surfaceContainer,
      child: Center(
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            if (userId != null)
              CircleAvatar(
                radius: 36,
                backgroundColor: colorForMxid(userId),
                child: Text(
                  initials(name),
                  style: const TextStyle(
                    fontSize: 24,
                    fontWeight: FontWeight.bold,
                    color: MatrixColors.surfaceContainer,
                  ),
                ),
              ),
            const SizedBox(height: 16),
            Text(
              name,
              style: const TextStyle(
                fontSize: 18,
                fontWeight: FontWeight.bold,
                color: MatrixColors.onContainer,
              ),
            ),
            if (userId != null)
              Padding(
                padding: const EdgeInsets.only(top: 4, bottom: 20),
                child: Text(
                  userId,
                  style: const TextStyle(color: MatrixColors.onContainerDim),
                ),
              ),
            ElevatedButton(onPressed: onLogout, child: const Text('Log out')),
          ],
        ),
      ),
    );
  }
}

// design/style.css's `.lobby-placeholder` tone — used for rail tabs that
// don't have a real screen behind them yet (no explore/inbox/search API
// on the Rust side, settings/feedback still just mockup dialogs).
class _ComingSoon extends StatelessWidget {
  const _ComingSoon();

  @override
  Widget build(BuildContext context) {
    return const ColoredBox(
      color: MatrixColors.surfaceContainer,
      child: Center(
        child: Text(
          'Not implemented yet',
          style: TextStyle(color: MatrixColors.onContainerDim, fontSize: 13),
        ),
      ),
    );
  }
}

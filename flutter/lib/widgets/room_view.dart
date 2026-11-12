import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:matrix_client/src/rust/api/matrix.dart';
import 'package:matrix_client/theme.dart';
import 'package:matrix_client/widgets/timeline_message.dart';

// The open room: header, grouped timeline, composer. No topic/action
// icons (design/style.css's room__topic/room__actions) — those are all
// for features not built yet (search, pins, calls). The members toggle
// is built (see MemberDrawer).
class RoomView extends StatelessWidget {
  const RoomView({
    super.key,
    required this.room,
    required this.messages,
    required this.messageController,
    required this.onSend,
    required this.busy,
    required this.membersOpen,
    required this.onToggleMembers,
  });

  final RoomSummary? room;
  final List<TimelineMessage> messages;
  final TextEditingController messageController;
  final VoidCallback onSend;
  final bool busy;
  final bool membersOpen;
  final VoidCallback onToggleMembers;

  @override
  Widget build(BuildContext context) {
    if (room == null) {
      return const Expanded(
        child: ColoredBox(
          color: MatrixColors.surfaceContainer,
          child: Center(
            child: Text(
              'Select a room',
              style: TextStyle(color: MatrixColors.onContainerDim),
            ),
          ),
        ),
      );
    }

    return Expanded(
      child: ColoredBox(
        color: MatrixColors.surfaceContainer,
        child: Column(
          children: [
            _Header(
              room: room!,
              membersOpen: membersOpen,
              onToggleMembers: onToggleMembers,
            ),
            const Divider(height: 1, color: MatrixColors.bgContainerLine),
            Expanded(child: _Timeline(messages: messages)),
            _Composer(
              controller: messageController,
              onSend: busy ? null : onSend,
            ),
          ],
        ),
      ),
    );
  }
}

class _Header extends StatelessWidget {
  const _Header({
    required this.room,
    required this.membersOpen,
    required this.onToggleMembers,
  });

  final RoomSummary room;
  final bool membersOpen;
  final VoidCallback onToggleMembers;

  @override
  Widget build(BuildContext context) {
    return SizedBox(
      height: 58,
      child: Row(
        children: [
          const SizedBox(width: 16),
          Container(
            width: 36,
            height: 36,
            decoration: const BoxDecoration(
              color: MatrixColors.secondaryContainer,
              shape: BoxShape.circle,
            ),
            alignment: Alignment.center,
            child: const Icon(
              Icons.tag,
              size: 17,
              color: MatrixColors.secondaryOnContainer,
            ),
          ),
          const SizedBox(width: 12),
          Expanded(
            child: Text(
              room.displayName,
              overflow: TextOverflow.ellipsis,
              style: const TextStyle(
                fontSize: 15.5,
                fontWeight: FontWeight.bold,
                color: MatrixColors.onContainer,
              ),
            ),
          ),
          IconButton(
            tooltip: 'Show Members',
            color: membersOpen
                ? MatrixColors.primaryMain
                : MatrixColors.onContainerDim,
            onPressed: onToggleMembers,
            icon: const Icon(Icons.people_outline),
          ),
          const SizedBox(width: 8),
        ],
      ),
    );
  }
}

class _Timeline extends StatelessWidget {
  const _Timeline({required this.messages});

  final List<TimelineMessage> messages;

  @override
  Widget build(BuildContext context) {
    // Not a ListView.builder: a date divider needs to know about the
    // previous message's day, which `itemBuilder`'s per-index callback
    // can't cheaply track — same non-builder pattern RoomList already
    // uses for its (similarly bounded) room list.
    final children = <Widget>[];
    DateTime? lastDay;
    for (var i = 0; i < messages.length; i++) {
      final message = messages[i];
      final sentAt = DateTime.fromMillisecondsSinceEpoch(
        message.timestampMs.toInt(),
      ).toLocal();
      final day = DateTime(sentAt.year, sentAt.month, sentAt.day);
      final isNewDay = lastDay == null || day != lastDay;
      if (isNewDay) {
        children.add(_DateDivider(day: day));
        lastDay = day;
      }
      // A day boundary always starts a fresh header, even for the same
      // sender as the last message of the previous day.
      final collapsed =
          !isNewDay && i > 0 && messages[i - 1].sender == message.sender;
      children.add(
        TimelineMessageRow(message: message, showHeader: !collapsed),
      );
    }
    return ListView(
      padding: const EdgeInsets.symmetric(vertical: 16),
      children: children,
    );
  }
}

// "Today"/"Yesterday"/"D Mon YYYY" — same manual-formatting approach as
// timeline_message.dart's `_formatTime`, no `intl` dependency for this.
class _DateDivider extends StatelessWidget {
  const _DateDivider({required this.day});

  final DateTime day;

  static const _months = [
    'Jan', 'Feb', 'Mar', 'Apr', 'May', 'Jun',
    'Jul', 'Aug', 'Sep', 'Oct', 'Nov', 'Dec',
  ];

  String get _label {
    final now = DateTime.now();
    final today = DateTime(now.year, now.month, now.day);
    final yesterday = today.subtract(const Duration(days: 1));
    if (day == today) return 'Today';
    if (day == yesterday) return 'Yesterday';
    return '${day.day} ${_months[day.month - 1]} ${day.year}';
  }

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 10, horizontal: 18),
      child: Row(
        children: [
          const Expanded(
            child: Divider(height: 1, color: MatrixColors.bgContainerLine),
          ),
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 10),
            child: Text(
              _label,
              style: const TextStyle(
                color: MatrixColors.onContainerFaint,
                fontSize: 11.5,
                fontWeight: FontWeight.bold,
              ),
            ),
          ),
          const Expanded(
            child: Divider(height: 1, color: MatrixColors.bgContainerLine),
          ),
        ],
      ),
    );
  }
}

class _Composer extends StatelessWidget {
  const _Composer({required this.controller, required this.onSend});

  final TextEditingController controller;
  final VoidCallback? onSend;

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.fromLTRB(16, 8, 16, 16),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.end,
        children: [
          Expanded(
            child: Container(
              constraints: const BoxConstraints(minHeight: 48, maxHeight: 160),
              decoration: BoxDecoration(
                color: MatrixColors.surfaceVariantContainer,
                borderRadius: BorderRadius.circular(10),
                border: Border.all(color: MatrixColors.surfaceVariantLine),
              ),
              child: Focus(
                // Enter sends; Shift+Enter inserts a newline (the
                // multiline TextField's default, so just let it through).
                onKeyEvent: (node, event) {
                  final isEnter =
                      event.logicalKey == LogicalKeyboardKey.enter;
                  if (event is KeyDownEvent &&
                      isEnter &&
                      !HardwareKeyboard.instance.isShiftPressed) {
                    onSend?.call();
                    return KeyEventResult.handled;
                  }
                  return KeyEventResult.ignored;
                },
                child: TextField(
                  controller: controller,
                  minLines: 1,
                  maxLines: 6,
                  style: const TextStyle(
                    color: MatrixColors.onContainer,
                    fontSize: 14,
                  ),
                  decoration: const InputDecoration(
                    border: InputBorder.none,
                    contentPadding: EdgeInsets.symmetric(
                      horizontal: 14,
                      vertical: 13,
                    ),
                    hintText: 'Message',
                    hintStyle: TextStyle(color: MatrixColors.onContainerFaint),
                  ),
                ),
              ),
            ),
          ),
          const SizedBox(width: 6),
          IconButton(
            color: MatrixColors.onContainerDim,
            onPressed: onSend,
            icon: const Icon(Icons.send),
          ),
        ],
      ),
    );
  }
}

import 'package:flutter/material.dart';
import 'package:matrix_client/src/rust/api/matrix.dart';
import 'package:matrix_client/theme.dart';

// Sidebar rail (design/index.html's <nav class="rail">). Only `home` and
// a selected space have real screens behind them today — matrix-client
// has no DM/explore/inbox/search API yet, and settings/search/feedback
// are still just dialogs in the mockup. Those other tabs are wired (tap,
// active indicator) but route to a placeholder rather than fabricating
// content that isn't backed by real data.
enum RailTab { home, direct, explore, addSpace, feedback, search, inbox, settings }

class RailNav extends StatelessWidget {
  const RailNav({
    super.key,
    required this.activeTab,
    required this.spaces,
    required this.activeSpaceId,
    required this.userId,
    required this.onSelect,
    required this.onSelectSpace,
  });

  final RailTab? activeTab;
  final List<SpaceSummary> spaces;
  final String? activeSpaceId;
  final String? userId;
  final ValueChanged<RailTab> onSelect;
  final ValueChanged<SpaceSummary> onSelectSpace;

  @override
  Widget build(BuildContext context) {
    return Container(
      width: 66,
      decoration: const BoxDecoration(
        color: MatrixColors.bgContainer,
        border: Border(
          right: BorderSide(color: MatrixColors.bgContainerLine),
        ),
      ),
      child: Column(
        children: [
          const SizedBox(height: 10),
          Expanded(
            child: SingleChildScrollView(
              child: Column(
                children: [
                  _RailStack(
                    children: [
                      _RailItem(
                        icon: Icons.home_rounded,
                        tooltip: 'Home',
                        active: activeTab == RailTab.home,
                        onTap: () => onSelect(RailTab.home),
                      ),
                      _RailItem(
                        icon: Icons.person_outline,
                        tooltip: 'Direct Messages',
                        active: activeTab == RailTab.direct,
                        onTap: () => onSelect(RailTab.direct),
                      ),
                    ],
                  ),
                  if (spaces.isNotEmpty) ...[
                    const _RailSep(),
                    _RailStack(
                      children: [
                        for (final space in spaces)
                          _RailItem(
                            tooltip: space.displayName,
                            active: activeSpaceId == space.roomId,
                            colorSeed: space.roomId,
                            avatarText: initials(space.displayName),
                            onTap: () => onSelectSpace(space),
                          ),
                      ],
                    ),
                  ],
                  const _RailSep(),
                  _RailStack(
                    children: [
                      _RailItem(
                        icon: Icons.explore_outlined,
                        tooltip: 'Explore Community',
                        active: activeTab == RailTab.explore,
                        onTap: () => onSelect(RailTab.explore),
                      ),
                      _RailItem(
                        icon: Icons.add,
                        tooltip: 'Add Space',
                        active: activeTab == RailTab.addSpace,
                        onTap: () => onSelect(RailTab.addSpace),
                      ),
                    ],
                  ),
                ],
              ),
            ),
          ),
          const _RailSep(),
          _RailStack(
            bottomPadded: true,
            children: [
              _RailItem(
                icon: Icons.star_border_rounded,
                tooltip: 'Leave Feedback',
                active: activeTab == RailTab.feedback,
                onTap: () => onSelect(RailTab.feedback),
              ),
              _RailItem(
                icon: Icons.search,
                tooltip: 'Search',
                active: activeTab == RailTab.search,
                onTap: () => onSelect(RailTab.search),
              ),
              _RailItem(
                icon: Icons.mail_outline,
                tooltip: 'Inbox',
                active: activeTab == RailTab.inbox,
                onTap: () => onSelect(RailTab.inbox),
              ),
              _RailItem(
                tooltip: 'User Settings',
                active: activeTab == RailTab.settings,
                colorSeed: userId,
                avatarText: userId == null
                    ? null
                    : initials(userId!.replaceFirst('@', '')),
                onTap: () => onSelect(RailTab.settings),
              ),
            ],
          ),
        ],
      ),
    );
  }
}

class _RailStack extends StatelessWidget {
  const _RailStack({required this.children, this.bottomPadded = false});

  final List<Widget> children;
  final bool bottomPadded;

  @override
  Widget build(BuildContext context) {
    final spaced = <Widget>[];
    for (final child in children) {
      if (spaced.isNotEmpty) spaced.add(const SizedBox(height: 8));
      spaced.add(child);
    }
    return Padding(
      padding: EdgeInsets.fromLTRB(0, 8, 0, bottomPadded ? 10 : 8),
      child: Column(children: spaced),
    );
  }
}

class _RailSep extends StatelessWidget {
  const _RailSep();

  @override
  Widget build(BuildContext context) {
    return const Padding(
      padding: EdgeInsets.symmetric(vertical: 2),
      child: Center(
        child: SizedBox(
          width: 32,
          height: 1,
          child: ColoredBox(color: MatrixColors.bgContainerLine),
        ),
      ),
    );
  }
}

// A rail nav entry. Either an outlined icon (nav tabs with no backing
// screen yet) or a colored initials circle (`colorSeed`/`avatarText` set
// — used for both the real joined spaces and the logged-in user, same
// visual treatment design/style.css gives both via `.rail-avatar[style*=
// "--mx"]`).
class _RailItem extends StatelessWidget {
  const _RailItem({
    this.icon,
    required this.tooltip,
    required this.active,
    required this.onTap,
    this.colorSeed,
    this.avatarText,
  });

  final IconData? icon;
  final String tooltip;
  final bool active;
  final VoidCallback onTap;
  final String? colorSeed;
  final String? avatarText;

  @override
  Widget build(BuildContext context) {
    final Widget avatar;
    if (colorSeed != null) {
      avatar = CircleAvatar(
        radius: 21,
        backgroundColor: colorForMxid(colorSeed!),
        child: Text(
          avatarText ?? '?',
          style: const TextStyle(
            color: MatrixColors.surfaceContainer,
            fontWeight: FontWeight.bold,
            fontSize: 15,
          ),
        ),
      );
    } else {
      avatar = Container(
        width: 42,
        height: 42,
        decoration: BoxDecoration(
          shape: BoxShape.circle,
          border: Border.all(color: MatrixColors.bgContainerLine),
        ),
        alignment: Alignment.center,
        child: Icon(
          icon,
          size: 20,
          color: active ? MatrixColors.primaryMain : MatrixColors.onContainerDim,
        ),
      );
    }

    return Tooltip(
      message: tooltip,
      child: SizedBox(
        width: double.infinity,
        height: 42,
        child: Stack(
          alignment: Alignment.centerLeft,
          children: [
            if (active)
              Container(
                width: 4,
                height: 24,
                decoration: const BoxDecoration(
                  color: MatrixColors.primaryMain,
                  borderRadius: BorderRadius.horizontal(
                    right: Radius.circular(4),
                  ),
                ),
              ),
            Center(
              child: Material(
                color: Colors.transparent,
                shape: const CircleBorder(),
                child: InkWell(
                  customBorder: const CircleBorder(),
                  onTap: onTap,
                  hoverColor: MatrixColors.bgContainerHover,
                  child: avatar,
                ),
              ),
            ),
          ],
        ),
      ),
    );
  }
}

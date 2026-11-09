import 'package:flutter/material.dart';
import 'package:matrix_client/src/rust/api/matrix.dart';
import 'package:matrix_client/theme.dart';

// Member list drawer for the open room (design/index.html's
// #membersDrawer, design/style.css's .members/.member-row). Grouped
// into "Admins"/"Members" by power level (`RoomMemberSummary.isAdmin`,
// moderator+ on the Rust side) — the mockup only has two buckets, not a
// finer admin/moderator split.
class MemberDrawer extends StatelessWidget {
  const MemberDrawer({super.key, required this.members, required this.busy});

  final List<RoomMemberSummary> members;
  final bool busy;

  @override
  Widget build(BuildContext context) {
    final admins = members.where((m) => m.isAdmin).toList();
    final regular = members.where((m) => !m.isAdmin).toList();

    return Container(
      width: 266,
      decoration: const BoxDecoration(
        color: MatrixColors.bgContainer,
        border: Border(left: BorderSide(color: MatrixColors.bgContainerLine)),
      ),
      child: Column(
        children: [
          const SizedBox(
            height: 58,
            child: Padding(
              padding: EdgeInsets.symmetric(horizontal: 16),
              child: Align(
                alignment: Alignment.centerLeft,
                child: Text(
                  'People',
                  style: TextStyle(
                    fontWeight: FontWeight.bold,
                    fontSize: 14,
                    color: MatrixColors.onContainer,
                  ),
                ),
              ),
            ),
          ),
          const Divider(height: 1, color: MatrixColors.bgContainerLine),
          Expanded(
            child: busy && members.isEmpty
                ? const Center(
                    child: SizedBox(
                      width: 20,
                      height: 20,
                      child: CircularProgressIndicator(
                        strokeWidth: 2,
                        color: MatrixColors.primaryMain,
                      ),
                    ),
                  )
                : ListView(
                    padding: const EdgeInsets.symmetric(vertical: 8),
                    children: [
                      if (admins.isNotEmpty) ...[
                        _GroupLabel('Admins — ${admins.length}'),
                        for (final member in admins) _MemberRow(member: member),
                      ],
                      if (regular.isNotEmpty) ...[
                        _GroupLabel('Members — ${regular.length}'),
                        for (final member in regular) _MemberRow(member: member),
                      ],
                    ],
                  ),
          ),
        ],
      ),
    );
  }
}

class _GroupLabel extends StatelessWidget {
  const _GroupLabel(this.text);

  final String text;

  @override
  Widget build(BuildContext context) {
    return Padding(
      padding: const EdgeInsets.fromLTRB(16, 14, 16, 6),
      child: Text(
        text.toUpperCase(),
        style: const TextStyle(
          fontSize: 11.5,
          fontWeight: FontWeight.bold,
          letterSpacing: 0.4,
          color: MatrixColors.onContainerFaint,
        ),
      ),
    );
  }
}

class _MemberRow extends StatelessWidget {
  const _MemberRow({required this.member});

  final RoomMemberSummary member;

  @override
  Widget build(BuildContext context) {
    return Material(
      color: Colors.transparent,
      child: InkWell(
        hoverColor: MatrixColors.bgContainerHover,
        onTap: () {},
        child: Padding(
          padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 6),
          child: Row(
            children: [
              CircleAvatar(
                radius: 14,
                backgroundColor: colorForMxid(member.userId),
                child: Text(
                  initials(member.displayName),
                  style: const TextStyle(
                    fontSize: 11,
                    fontWeight: FontWeight.bold,
                    color: MatrixColors.surfaceContainer,
                  ),
                ),
              ),
              const SizedBox(width: 10),
              Expanded(
                child: Text(
                  member.displayName,
                  overflow: TextOverflow.ellipsis,
                  style: const TextStyle(
                    fontSize: 13.5,
                    color: MatrixColors.onContainerDim,
                  ),
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}

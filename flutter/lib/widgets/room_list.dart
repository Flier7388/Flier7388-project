import 'package:flutter/material.dart';
import 'package:matrix_client/src/rust/api/matrix.dart';
import 'package:matrix_client/theme.dart';

// Cinny-style room list rows (design/style.css's `.room-row`). No unread
// pills, no categories — RoomSummary doesn't carry that data yet.
class RoomList extends StatelessWidget {
  const RoomList({
    super.key,
    this.title = 'Rooms',
    required this.rooms,
    required this.selectedRoomId,
    required this.onSelect,
    required this.onRefresh,
    required this.busy,
  });

  final String title;
  final List<RoomSummary> rooms;
  final String? selectedRoomId;
  final ValueChanged<RoomSummary> onSelect;
  final VoidCallback onRefresh;
  final bool busy;

  @override
  Widget build(BuildContext context) {
    return Container(
      width: 290,
      color: MatrixColors.bgContainer,
      child: Column(
        children: [
          SizedBox(
            height: 58,
            child: Row(
              children: [
                const SizedBox(width: 16),
                Expanded(
                  child: Text(
                    title,
                    overflow: TextOverflow.ellipsis,
                    style: const TextStyle(
                      fontSize: 16,
                      fontWeight: FontWeight.bold,
                      color: MatrixColors.onContainer,
                    ),
                  ),
                ),
                IconButton(
                  tooltip: 'Refresh rooms',
                  color: MatrixColors.onContainerDim,
                  onPressed: busy ? null : onRefresh,
                  icon: const Icon(Icons.refresh),
                ),
                const SizedBox(width: 8),
              ],
            ),
          ),
          const Divider(height: 1, color: MatrixColors.bgContainerLine),
          Expanded(
            child: ListView(
              padding: const EdgeInsets.all(8),
              children: rooms.map((room) {
                final selected = room.roomId == selectedRoomId;
                return _RoomRow(
                  room: room,
                  selected: selected,
                  onTap: () => onSelect(room),
                );
              }).toList(),
            ),
          ),
        ],
      ),
    );
  }
}

class _RoomRow extends StatelessWidget {
  const _RoomRow({
    required this.room,
    required this.selected,
    required this.onTap,
  });

  final RoomSummary room;
  final bool selected;
  final VoidCallback onTap;

  @override
  Widget build(BuildContext context) {
    return Material(
      color: selected
          ? MatrixColors.bgContainerActive
          : Colors.transparent,
      borderRadius: BorderRadius.circular(8),
      child: InkWell(
        onTap: onTap,
        borderRadius: BorderRadius.circular(8),
        hoverColor: MatrixColors.bgContainerHover,
        child: Padding(
          padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 7),
          child: Row(
            children: [
              Container(
                width: 30,
                height: 30,
                decoration: BoxDecoration(
                  color: MatrixColors.secondaryContainer,
                  borderRadius: BorderRadius.circular(9),
                ),
                alignment: Alignment.center,
                child: const Icon(
                  Icons.tag,
                  size: 15,
                  color: MatrixColors.secondaryOnContainer,
                ),
              ),
              const SizedBox(width: 10),
              Expanded(
                child: Text(
                  room.displayName,
                  overflow: TextOverflow.ellipsis,
                  style: TextStyle(
                    fontSize: 13.5,
                    color: selected
                        ? MatrixColors.onContainer
                        : MatrixColors.onContainerDim,
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

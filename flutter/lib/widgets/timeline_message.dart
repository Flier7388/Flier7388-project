import 'package:flutter/material.dart';
import 'package:matrix_client/src/rust/api/matrix.dart';
import 'package:matrix_client/theme.dart';

// One timeline row. [showHeader] is false for messages that follow
// another message from the same sender — mirrors design/style.css's
// `.msg.is-collapsed` (consecutive-same-sender grouping, matching
// design/script.js's own `collapsed = messages[i-1].author === m.author`).
class TimelineMessageRow extends StatelessWidget {
  const TimelineMessageRow({
    super.key,
    required this.message,
    required this.showHeader,
  });

  final TimelineMessage message;
  final bool showHeader;

  @override
  Widget build(BuildContext context) {
    final color = colorForMxid(message.sender);
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 18, vertical: 3),
      child: Row(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          SizedBox(
            width: 36,
            child: showHeader
                ? CircleAvatar(
                    radius: 17,
                    backgroundColor: color,
                    child: Text(
                      initials(message.sender),
                      style: const TextStyle(
                        color: MatrixColors.surfaceContainer,
                        fontWeight: FontWeight.bold,
                        fontSize: 13,
                      ),
                    ),
                  )
                : null,
          ),
          const SizedBox(width: 12),
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                if (showHeader)
                  Padding(
                    padding: const EdgeInsets.only(bottom: 1),
                    child: Row(
                      crossAxisAlignment: CrossAxisAlignment.baseline,
                      textBaseline: TextBaseline.alphabetic,
                      children: [
                        Text(
                          message.sender,
                          style: TextStyle(
                            color: color,
                            fontWeight: FontWeight.bold,
                            fontSize: 14,
                          ),
                        ),
                        const SizedBox(width: 6),
                        Text(
                          _formatTime(message.timestampMs),
                          style: const TextStyle(
                            color: MatrixColors.onContainerFaint,
                            fontSize: 11.5,
                          ),
                        ),
                      ],
                    ),
                  ),
                Text(
                  message.body,
                  style: const TextStyle(
                    color: MatrixColors.onContainer,
                    fontSize: 14,
                    height: 1.5,
                  ),
                ),
              ],
            ),
          ),
        ],
      ),
    );
  }
}

// Local time, "HH:MM" — the day/month/year is the `_DateDivider`
// (room_view.dart)'s job, not this per-message label's. Done by hand
// rather than pulling in `intl` for two zero-padded ints.
String _formatTime(BigInt timestampMs) {
  final time = DateTime.fromMillisecondsSinceEpoch(
    timestampMs.toInt(),
  ).toLocal();
  final hour = time.hour.toString().padLeft(2, '0');
  final minute = time.minute.toString().padLeft(2, '0');
  return '$hour:$minute';
}

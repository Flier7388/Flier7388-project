import 'package:flutter/material.dart';

// Color tokens ported 1:1 from design/style.css's :root (cinny's real
// dark-theme values, not approximated).
class MatrixColors {
  static const bgContainer = Color(0xFF1A1A1A);
  static const bgContainerHover = Color(0xFF262626);
  static const bgContainerActive = Color(0xFF333333);
  static const bgContainerLine = Color(0xFF404040);
  static const onContainer = Color(0xFFF2F2F2);
  static const onContainerDim = Color(0x99F2F2F2); // 60%
  static const onContainerFaint = Color(0x61F2F2F2); // 38%

  static const surfaceContainer = Color(0xFF262626);
  static const surfaceContainerHover = Color(0xFF333333);
  static const surfaceContainerActive = Color(0xFF404040);
  static const surfaceContainerLine = Color(0xFF4D4D4D);

  static const surfaceVariantContainer = Color(0xFF333333);
  static const surfaceVariantLine = Color(0xFF595959);

  static const primaryMain = Color(0xFFBDB6EC);
  static const primaryContainer = Color(0xFF413C65);
  static const primaryOnContainer = Color(0xFFE3E1F7);

  static const secondaryContainer = Color(0xFF404040);
  static const secondaryOnContainer = Color(0xFFF2F2F2);

  static const criticalMain = Color(0xFFE69D9D);
  static const criticalContainer = Color(0xFF602929);

  static const successMain = Color(0xFF85E0BA);

  // colorMXID.js: 8 colors, picked by hashCode(userId) % 8.
  static const mxUc = <Color>[
    Color(0xFF80BFFF), // hsl(208,100%,75%)
    Color(0xFFFF80FF), // hsl(301,100%,80%)
    Color(0xFF66FFC2), // hsl(163,100%,70%)
    Color(0xFFFF80AD), // hsl(343,100%,75%)
    Color(0xFFFFA366), // hsl(24,100%,70%)
    Color(0xFF33FFF0), // hsl(181,100%,60%)
    Color(0xFFA3A6FF), // hsl(243,100%,80%)
    Color(0xFFC2FF80), // hsl(94,100%,80%)
  ];
}

// Ported verbatim from design/script.js's hashCode()/colorMXID() — same
// hash, same `% 8`, so the same user ID always maps to the same color.
int _hashCode(String str) {
  var hash = 0;
  for (var i = 0; i < str.length; i++) {
    hash = (hash << 5) - hash + str.codeUnitAt(i);
    hash |= 0;
  }
  return hash.abs();
}

Color colorForMxid(String userId) {
  final n = _hashCode(userId) % MatrixColors.mxUc.length;
  return MatrixColors.mxUc[n];
}

String initials(String name) =>
    name.trim().isEmpty ? '?' : name.trim()[0].toUpperCase();

ThemeData buildMatrixTheme() {
  return ThemeData(
    brightness: Brightness.dark,
    scaffoldBackgroundColor: MatrixColors.bgContainer,
    colorScheme: const ColorScheme.dark(
      surface: MatrixColors.bgContainer,
      primary: MatrixColors.primaryMain,
      onPrimary: MatrixColors.primaryOnContainer,
      error: MatrixColors.criticalMain,
    ),
    textTheme: const TextTheme(
      bodyMedium: TextStyle(color: MatrixColors.onContainer, fontSize: 14),
    ),
    inputDecorationTheme: InputDecorationTheme(
      filled: true,
      fillColor: MatrixColors.surfaceVariantContainer,
      labelStyle: const TextStyle(color: MatrixColors.onContainerDim),
      border: OutlineInputBorder(
        borderRadius: BorderRadius.circular(10),
        borderSide: const BorderSide(color: MatrixColors.surfaceVariantLine),
      ),
    ),
    elevatedButtonTheme: ElevatedButtonThemeData(
      style: ElevatedButton.styleFrom(
        backgroundColor: MatrixColors.primaryContainer,
        foregroundColor: MatrixColors.primaryOnContainer,
      ),
    ),
  );
}

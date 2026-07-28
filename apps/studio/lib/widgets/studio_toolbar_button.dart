import 'package:flutter/material.dart';
import 'package:hugeicons/hugeicons.dart';

/// Tombol toolbar bergaya ghost/outlined yang cocok dengan tema dark IDE.
///
/// Digunakan di berbagai tempat seperti toolbar database, editor, dll.
/// Menampilkan icon + label dengan warna subtle yang transparan.
class StudioToolbarButton extends StatelessWidget {
  /// Data icon dari HugeIcons (tipe `List<List<dynamic>>`).
  final List<List<dynamic>> icon;

  /// Label teks yang ditampilkan di samping icon.
  final String label;

  /// Warna aksen untuk icon, teks, border, dan background.
  final Color color;

  /// Callback saat tombol ditekan.
  final VoidCallback onPressed;

  /// Ukuran icon. Default: 13.
  final double iconSize;

  /// Ukuran font label. Default: 11.
  final double fontSize;

  const StudioToolbarButton({
    super.key,
    required this.icon,
    required this.label,
    required this.color,
    required this.onPressed,
    this.iconSize = 13,
    this.fontSize = 11,
  });

  @override
  Widget build(BuildContext context) {
    return Material(
      color: Colors.transparent,
      child: InkWell(
        borderRadius: BorderRadius.circular(6),
        onTap: onPressed,
        child: Container(
          padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 5),
          decoration: BoxDecoration(
            borderRadius: BorderRadius.circular(6),
            border: Border.all(color: color.withAlpha(80)),
            color: color.withAlpha(20),
          ),
          child: Row(
            mainAxisSize: MainAxisSize.min,
            children: [
              HugeIcon(icon: icon, size: iconSize, color: color),
              const SizedBox(width: 5),
              Text(
                label,
                style: TextStyle(
                  color: color,
                  fontSize: fontSize,
                  fontWeight: FontWeight.w500,
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}

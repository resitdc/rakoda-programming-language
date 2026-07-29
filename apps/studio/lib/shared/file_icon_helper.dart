import 'package:flutter/material.dart';
import 'package:flutter_svg/flutter_svg.dart';
import 'package:hugeicons/hugeicons.dart';

class FileIconHelper {
  static Widget getFileIcon(String fileName, {double size = 16}) {
    final lowerName = fileName.toLowerCase();
    
    // Check for SVG icons first
    if (lowerName.endsWith('.php')) return SvgPicture.asset('assets/icons/php.svg', width: size, height: size);
    if (lowerName.endsWith('.js')) return SvgPicture.asset('assets/icons/javascript.svg', width: size, height: size);
    if (lowerName.endsWith('.node')) return SvgPicture.asset('assets/icons/nodejs.svg', width: size, height: size);
    if (lowerName.endsWith('.py')) return SvgPicture.asset('assets/icons/python.svg', width: size, height: size);
    if (lowerName.endsWith('.rs')) return SvgPicture.asset('assets/icons/rust.svg', width: size, height: size);
    if (lowerName.endsWith('.java')) return SvgPicture.asset('assets/icons/java.svg', width: size, height: size);
    if (lowerName.endsWith('.json')) return SvgPicture.asset('assets/icons/json.svg', width: size, height: size);
    if (lowerName.endsWith('.css')) return SvgPicture.asset('assets/icons/css.svg', width: size, height: size);
    if (lowerName.endsWith('.html')) return SvgPicture.asset('assets/icons/html.svg', width: size, height: size);

    // Fallback to IconData
    IconData iconData = Icons.insert_drive_file;
    Color color = Colors.white70;

    if (lowerName.endsWith('.rpl')) {
      iconData = Icons.code;
      color = const Color(0xFF4EC9B0);
    } else if (lowerName.endsWith('.db') || lowerName.endsWith('.sqlite')) {
      iconData = Icons.storage;
      color = Colors.yellow;
    } else if (lowerName.endsWith('.md')) {
      iconData = Icons.insert_drive_file;
      color = Colors.lightBlue;
    } else if (lowerName.endsWith('.txt')) {
      iconData = Icons.text_snippet;
      color = Colors.white54;
    } else if (lowerName.endsWith('.pdf')) {
      iconData = Icons.picture_as_pdf;
      color = Colors.redAccent;
    } else if (lowerName.endsWith('.png') || lowerName.endsWith('.jpg') || lowerName.endsWith('.jpeg')) {
      iconData = Icons.image;
      color = Colors.purpleAccent;
    }

    return Icon(iconData, size: size, color: color);
  }
}

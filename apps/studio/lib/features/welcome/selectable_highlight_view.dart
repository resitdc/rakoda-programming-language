import 'package:flutter/material.dart';
import 'package:highlight/highlight.dart' show highlight, Node;

class SelectableHighlightView extends StatelessWidget {
  final String source;
  final String? language;
  final Map<String, TextStyle> theme;
  final EdgeInsetsGeometry? padding;
  final TextStyle? textStyle;
  final int tabSize;

  SelectableHighlightView(
    String input, {
    super.key,
    this.language,
    this.theme = const {},
    this.padding,
    this.textStyle,
    this.tabSize = 8,
  }) : source = input;

  List<TextSpan> _convert(List<Node> nodes) {
    List<TextSpan> spans = [];
    var currentSpans = spans;
    List<List<TextSpan>> stack = [];

    void traverse(Node node) {
      if (node.value != null) {
        currentSpans.add(node.className == null
            ? TextSpan(text: node.value)
            : TextSpan(text: node.value, style: theme[node.className!]));
      } else if (node.children != null) {
        List<TextSpan> tmp = [];
        currentSpans.add(TextSpan(children: tmp, style: theme[node.className!]));
        stack.add(currentSpans);
        currentSpans = tmp;

        for (var n in node.children!) {
          traverse(n);
          if (n == node.children!.last) {
            currentSpans = stack.isEmpty ? spans : stack.removeLast();
          }
        }
      }
    }

    for (var node in nodes) {
      traverse(node);
    }

    return spans;
  }

  @override
  Widget build(BuildContext context) {
    var style = TextStyle(
      fontFamily: 'monospace',
      color: theme['root']?.color ?? const Color(0xff000000),
    );
    if (textStyle != null) {
      style = style.merge(textStyle);
    }

    return Container(
      color: theme['root']?.backgroundColor ?? const Color(0xffffffff),
      padding: padding,
      child: Text.rich(
        TextSpan(
          style: style,
          children: _convert(highlight.parse(source, language: language).nodes!),
        ),
      ),
    );
  }
}
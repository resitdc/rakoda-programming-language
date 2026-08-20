import 'dart:io';
import 'dart:async';
import 'dart:math';
import 'package:flutter/material.dart';
import 'package:hugeicons/hugeicons.dart';
import 'package:flutter/services.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../settings/settings_provider.dart';
import 'package:flutter_code_editor/flutter_code_editor.dart';
import 'package:flutter_highlight/themes/vs2015.dart';
import 'package:flutter_highlight/themes/monokai.dart';
import 'package:flutter_highlight/themes/monokai-sublime.dart';
import 'package:flutter_highlight/themes/dracula.dart';
import 'package:flutter_highlight/themes/github.dart';
import 'package:flutter_highlight/themes/atom-one-dark.dart';
import 'package:highlight/highlight.dart';
import 'package:highlight/languages/css.dart';
import 'package:highlight/languages/javascript.dart';
import 'package:highlight/languages/xml.dart';
import 'package:highlight/languages/php.dart';
import 'package:highlight/languages/python.dart';
import 'package:highlight/languages/rust.dart';
import 'package:highlight/languages/json.dart';
import 'package:highlight/languages/java.dart';
import 'package:highlight/languages/typescript.dart';
import 'rpl_languages.dart';
import 'editor_tab.dart';
import '../../features/theme/theme_state.dart';
import '../../shared/file_icon_helper.dart';
import 'dart:async';

class KeyboardEventNotifier {
  static final StreamController<String> symbolStream = StreamController<String>.broadcast();
}

class CodeEditor extends ConsumerStatefulWidget {
  final EditorTab tab;
  final int? initialLineNumber;
  final String? searchQuery;
  final void Function(String path, String content)? onSave;
  final void Function(String path)? onClose;
  final void Function(String content, int? selStart, int? selEnd)? onChanged;

  const CodeEditor({
    super.key,
    required this.tab,
    this.initialLineNumber,
    this.searchQuery,
    this.onSave,
    this.onClose,
    this.onChanged,
  });

  @override
  ConsumerState<CodeEditor> createState() => _CodeEditorState();
}

class _CodeEditorState extends ConsumerState<CodeEditor> {
  late CodeController _controller;
  late FocusNode _focusNode;
  late StreamSubscription _symbolSub;
  String _content = '';
  Timer? _debounceTimer;
  Timer? _autoSaveTimer;
  int _currentLine = 0;

  @override
  void initState() {
    super.initState();
    _focusNode = FocusNode()..addListener(() {
      setState(() {});
    });
    
    _symbolSub = KeyboardEventNotifier.symbolStream.stream.listen(_onSymbol);
    
    _content = widget.tab.content;
    
    _controller = CodeController(
      text: _content,
      language: _getLanguageMode(widget.tab.filePath, _content),
      patternMap: widget.searchQuery != null && widget.searchQuery!.isNotEmpty
          ? {
              '(?i)${RegExp.escape(widget.searchQuery!)}': const TextStyle(
                backgroundColor: Color(0xFF623A18), // Find match highlight background
                color: Colors.white,
              ),
            }
          : null,
    );

    if (widget.initialLineNumber != null) {
      final lines = _content.split('\n');
      int offset = 0;
      final targetLine = widget.initialLineNumber!;
      for (int i = 0; i < targetLine - 1 && i < lines.length; i++) {
        offset += lines[i].length + 1; // +1 for the newline character
      }
      _controller.selection = TextSelection.collapsed(offset: offset);
    }

    _setupAutocomplete();

    _controller.addListener(_onTextChanged);
  }

  void _setupAutocomplete() {
    final language = _controller.language;
    if (language == xml || language == rplHtml) {
      _controller.autocompleter.setCustomWords([
        'div', 'span', 'html', 'body', 'head', 'title', 'meta', 'link', 'style', 'script',
        'h1', 'h2', 'h3', 'h4', 'h5', 'h6', 'p', 'br', 'hr', 'a', 'img', 'ul', 'ol', 'li',
        'table', 'tr', 'td', 'th', 'thead', 'tbody', 'tfoot', 'form', 'input', 'button',
        'select', 'option', 'textarea', 'label', 'iframe', 'canvas', 'svg', 'nav', 'header',
        'footer', 'main', 'section', 'article', 'aside', 'class', 'id', 'src', 'href', 'style', 'type',
        '<!--', '-->', 'DOCTYPE', 'html'
      ]);
    } else if (language == css) {
      _controller.autocompleter.setCustomWords([
        'color', 'background', 'background-color', 'font-size', 'font-family', 'font-weight',
        'margin', 'margin-top', 'margin-right', 'margin-bottom', 'margin-left',
        'padding', 'padding-top', 'padding-right', 'padding-bottom', 'padding-left',
        'border', 'border-radius', 'width', 'height', 'display', 'position', 'top', 'right',
        'bottom', 'left', 'z-index', 'flex', 'justify-content', 'align-items', 'grid',
        'box-shadow', 'opacity', 'transition', 'transform', 'cursor', 'overflow', 'hover', 'active',
        'important', 'rem', 'em', 'px', 'vh', 'vw', 'auto', 'none', 'block', 'inline', 'inline-block'
      ]);
    } else if (language == javascript) {
      _controller.autocompleter.setCustomWords([
        'function', 'const', 'let', 'var', 'if', 'else', 'for', 'while', 'switch', 'case',
        'break', 'return', 'import', 'export', 'default', 'class', 'extends', 'constructor',
        'super', 'this', 'async', 'await', 'try', 'catch', 'finally', 'throw', 'new', 'typeof',
        'console.log', 'document.getElementById', 'document.querySelector', 'addEventListener',
        'setTimeout', 'setInterval', 'Promise', 'fetch', 'Math', 'JSON.parse', 'JSON.stringify'
      ]);
    }
  }

  Mode? _getLanguageMode(String filePath, String content) {
    final isLowEndMode = ref.read(settingsProvider).isLowEndMode;
    if (isLowEndMode) {
      final lineCount = '\n'.allMatches(content).length + 1;
      if (lineCount > 800) {
        return null; // Disable syntax highlighting to save memory
      }
    }

    final ext = filePath.split('.').last.toLowerCase();
    if (filePath.endsWith('.rpl.html') || filePath.endsWith('.html')) {
      return rplHtml;
    }
    if (ext == 'rpl') return rpl;
    if (ext == 'js') return javascript;
    if (ext == 'css') return css;
    if (ext == 'php') return php;
    if (ext == 'py') return python;
    if (ext == 'rs') return rust;
    if (ext == 'json') return json;
    if (ext == 'java') return java;
    if (ext == 'ts' || ext == 'tsx') return typescript;
    if (ext == 'html' || ext == 'xml') return xml;
    return null;
  }

  void _onTextChanged() {
    final hasContentChanged = _controller.text != _content;
    
    setState(() {
      _content = _controller.text;
      widget.tab.content = _content;
      if (hasContentChanged) {
        widget.tab.isModified = true;
      }
    });

    // Selalu trigger onChanged karena seleksi (kursor) mungkin berubah
    widget.onChanged?.call(
      _content,
    _controller.selection.baseOffset,
      _controller.selection.extentOffset,
    );
    
    final selection = _controller.selection;
    if (selection.baseOffset >= 0) {
      final textBeforeCursor = _controller.text.substring(0, min(selection.baseOffset, _controller.text.length));
      final currentLine = textBeforeCursor.split('\n').length - 1;
      if (currentLine != _currentLine) {
        setState(() {
          _currentLine = currentLine;
        });
      }
    }
    
    if (hasContentChanged) {
      final settings = ref.read(settingsProvider);
      if (settings.isAutoSave) {
        _autoSaveTimer?.cancel();
        _autoSaveTimer = Timer(const Duration(milliseconds: 500), () {
          if (mounted) save();
        });
      }
      
      final isLowEndMode = settings.isLowEndMode;
      if (isLowEndMode) {
        if (_controller.language != null) {
          _controller.language = null; // Disable highlighting temporarily
        }
        
        _debounceTimer?.cancel();
        _debounceTimer = Timer(const Duration(milliseconds: 700), () {
          if (mounted) {
            _controller.language = _getLanguageMode(widget.tab.filePath, _content);
          }
        });
      }
    }
  }

  void save() {
    try {
      final file = File(widget.tab.filePath);
      file.writeAsStringSync(_content);
      widget.tab.isModified = false;
      setState(() {});
      widget.onSave?.call(widget.tab.filePath, _content);
    } catch (e) {
      ScaffoldMessenger.of(context)
          .showSnackBar(SnackBar(content: Text('Gagal menyimpan: $e')));
    }
  }

  void _onSymbol(String symbol) {
    if (_focusNode.hasFocus) {
      final selection = _controller.selection;
      if (selection.baseOffset >= 0 && selection.extentOffset >= 0) {
        final currentText = _controller.text;
        final newText = currentText.replaceRange(selection.start, selection.end, symbol);
        _controller.value = _controller.value.copyWith(
          text: newText,
          selection: TextSelection.collapsed(offset: selection.start + symbol.length),
        );
      }
    }
  }

  void _handleZoomIn() {
    final fontSize = ref.read(settingsProvider).editorFontSize;
    ref.read(settingsProvider.notifier).setEditorFontSize((fontSize + 1).clamp(8.0, 48.0));
  }

  void _handleZoomOut() {
    final fontSize = ref.read(settingsProvider).editorFontSize;
    ref.read(settingsProvider.notifier).setEditorFontSize((fontSize - 1).clamp(8.0, 48.0));
  }

  int _getOffsetForLine(List<String> lines, int lineIndex) {
    int offset = 0;
    for (int i = 0; i < lineIndex && i < lines.length; i++) {
      offset += lines[i].length + 1;
    }
    return offset;
  }

  void _moveLine(int dir) {
    if (!_focusNode.hasFocus) return;
    final selection = _controller.selection;
    if (!selection.isValid) return;

    final text = _controller.text;
    final lines = text.split('\n');
    int startLine = '\n'.allMatches(text.substring(0, selection.start)).length;
    int endLine = '\n'.allMatches(text.substring(0, selection.end)).length;

    if (dir == -1 && startLine > 0) {
      final lineAbove = lines.removeAt(startLine - 1);
      lines.insert(endLine, lineAbove);
    } else if (dir == 1 && endLine < lines.length - 1) {
      final lineBelow = lines.removeAt(endLine + 1);
      lines.insert(startLine, lineBelow);
    } else {
      return;
    }

    final newText = lines.join('\n');
    int newStart = _getOffsetForLine(lines, startLine + dir) + (selection.start - _getOffsetForLine(text.split('\n'), startLine));
    int newEnd = _getOffsetForLine(lines, endLine + dir) + (selection.end - _getOffsetForLine(text.split('\n'), endLine));

    _controller.value = _controller.value.copyWith(
      text: newText,
      selection: TextSelection(baseOffset: newStart, extentOffset: newEnd),
    );
  }

  void _duplicateLine(int dir) {
    if (!_focusNode.hasFocus) return;
    final selection = _controller.selection;
    if (!selection.isValid) return;

    final text = _controller.text;
    int lineStart = text.lastIndexOf('\n', selection.start - 1) + 1;
    int lineEnd = text.indexOf('\n', selection.end);
    if (lineEnd == -1) lineEnd = text.length;

    final selectedLinesText = text.substring(lineStart, lineEnd);
    final newText = text.replaceRange(lineEnd, lineEnd, '\n' + selectedLinesText);
    
    _controller.value = _controller.value.copyWith(
      text: newText,
      selection: dir == 1 ? TextSelection(baseOffset: selection.start + selectedLinesText.length + 1, extentOffset: selection.end + selectedLinesText.length + 1) : selection,
    );
  }

  void _deleteLine() {
    if (!_focusNode.hasFocus) return;
    final selection = _controller.selection;
    if (!selection.isValid) return;

    final text = _controller.text;
    int lineStart = text.lastIndexOf('\n', selection.start - 1) + 1;
    int lineEnd = text.indexOf('\n', selection.end);
    if (lineEnd == -1) lineEnd = text.length;

    // We also want to delete the trailing newline if it exists, or the leading newline if it's the last line.
    int removeEnd = lineEnd;
    if (removeEnd < text.length && text[removeEnd] == '\n') {
      removeEnd += 1;
    } else if (lineStart > 0 && text[lineStart - 1] == '\n') {
      lineStart -= 1;
    }

    final newText = text.replaceRange(lineStart, removeEnd, '');
    
    _controller.value = _controller.value.copyWith(
      text: newText,
      selection: TextSelection.collapsed(
        offset: lineStart.clamp(0, newText.length),
      ),
    );
  }

  void _indentSelection() {
    if (!_focusNode.hasFocus) return;
    final selection = _controller.selection;
    if (!selection.isValid) return;

    final text = _controller.text;
    int lineStart = text.lastIndexOf('\n', selection.start - 1) + 1;
    int lineEnd = text.indexOf('\n', selection.end);
    if (lineEnd == -1) lineEnd = text.length;

    final selectedText = text.substring(lineStart, lineEnd);
    final indentedText = selectedText.split('\n').map((line) => '  ' + line).join('\n');
    
    final newText = text.replaceRange(lineStart, lineEnd, indentedText);
    _controller.value = _controller.value.copyWith(
      text: newText,
      selection: TextSelection(
        baseOffset: selection.start + 2,
        extentOffset: selection.end + (indentedText.length - selectedText.length),
      ),
    );
  }

  void _outdentSelection() {
    if (!_focusNode.hasFocus) return;
    final selection = _controller.selection;
    if (!selection.isValid) return;

    final text = _controller.text;
    int lineStart = text.lastIndexOf('\n', selection.start - 1) + 1;
    int lineEnd = text.indexOf('\n', selection.end);
    if (lineEnd == -1) lineEnd = text.length;

    final selectedText = text.substring(lineStart, lineEnd);
    int removedCount = 0;
    int firstLineRemoved = 0;
    bool isFirst = true;

    final outdentedText = selectedText.split('\n').map((line) {
      if (line.startsWith('  ')) {
        removedCount += 2;
        if (isFirst) firstLineRemoved = 2;
        isFirst = false;
        return line.substring(2);
      } else if (line.startsWith(' ') || line.startsWith('\t')) {
        removedCount += 1;
        if (isFirst) firstLineRemoved = 1;
        isFirst = false;
        return line.substring(1);
      }
      isFirst = false;
      return line;
    }).join('\n');
    
    final newText = text.replaceRange(lineStart, lineEnd, outdentedText);
    _controller.value = _controller.value.copyWith(
      text: newText,
      selection: TextSelection(
        baseOffset: (selection.start - firstLineRemoved).clamp(lineStart, text.length),
        extentOffset: (selection.end - removedCount).clamp(lineStart, text.length),
      ),
    );
  }

  @override
  void dispose() {
    _debounceTimer?.cancel();
    _symbolSub.cancel();
    _focusNode.dispose();
    _controller.removeListener(_onTextChanged);
    _controller.dispose();
    super.dispose();
  }

  Map<String, TextStyle> _getTheme(String themeName) {
    switch (themeName) {
      case 'Monokai':
        return monokaiTheme;
      case 'Monokai Sublime':
        return monokaiSublimeTheme;
      case 'Dracula':
        return draculaTheme;
      case 'GitHub':
        return githubTheme;
      case 'Atom One Dark':
        return atomOneDarkTheme;
      case 'VS2015':
      default:
        return vs2015Theme;
    }
  }

  @override
  Widget build(BuildContext context) {
    final settings = ref.watch(settingsProvider);
    final isWordWrap = settings.isWordWrap;
    final editorFontSize = settings.editorFontSize;
    final baseTheme = _getTheme(settings.editorTheme);

    // We can inject custom background color into the theme map
    final customTheme = Map<String, TextStyle>.from(baseTheme);
    customTheme['root'] = customTheme['root']?.copyWith(
      backgroundColor: const Color(0xFF1E1E1E),
    ) ?? const TextStyle(backgroundColor: Color(0xFF1E1E1E));

    final codeTextStyle = TextStyle(
      fontFamily: 'monospace',
      fontSize: editorFontSize,
      height: 1.6,
    );

    final gutterTextStyle = TextStyle(
      color: const Color(0xFF6E7681),
      fontSize: editorFontSize,
      fontFamily: 'monospace',
      height: 1.6,
    );

    // VS Code style: active line number is bright, others are dim
    TextSpan lineNumberBuilder(int lineNumber, TextStyle? style) {
      final isActive = lineNumber == _currentLine + 1;
      return TextSpan(
        text: '$lineNumber',
        style: style?.copyWith(
          color: isActive ? const Color(0xFFC6C6C6) : const Color(0xFF6E7681),
        ),
      );
    }

    // Dynamic gutter width: scales with font size so numbers never get clipped
    // Formula: (charWidth * 3 digits) + margin(16) + internal subtraction(32)
    final gutterWidth = (editorFontSize * 0.6 * 3) + 16 + 32;

    return CallbackShortcuts(
      bindings: <ShortcutActivator, VoidCallback>{
        const SingleActivator(LogicalKeyboardKey.keyS, control: true): save,
        const SingleActivator(LogicalKeyboardKey.keyS, meta: true): save,
        const SingleActivator(LogicalKeyboardKey.equal, control: true): _handleZoomIn,
        const SingleActivator(LogicalKeyboardKey.equal, meta: true): _handleZoomIn,
        const SingleActivator(LogicalKeyboardKey.numpadAdd, control: true): _handleZoomIn,
        const SingleActivator(LogicalKeyboardKey.numpadAdd, meta: true): _handleZoomIn,
        const SingleActivator(LogicalKeyboardKey.minus, control: true): _handleZoomOut,
        const SingleActivator(LogicalKeyboardKey.minus, meta: true): _handleZoomOut,
        const SingleActivator(LogicalKeyboardKey.numpadSubtract, control: true): _handleZoomOut,
        const SingleActivator(LogicalKeyboardKey.numpadSubtract, meta: true): _handleZoomOut,
        const SingleActivator(LogicalKeyboardKey.bracketRight, control: true): _indentSelection,
        const SingleActivator(LogicalKeyboardKey.bracketRight, meta: true): _indentSelection,
        const SingleActivator(LogicalKeyboardKey.bracketLeft, control: true): _outdentSelection,
        const SingleActivator(LogicalKeyboardKey.bracketLeft, meta: true): _outdentSelection,
        const SingleActivator(LogicalKeyboardKey.arrowUp, alt: true): () => _moveLine(-1),
        const SingleActivator(LogicalKeyboardKey.arrowDown, alt: true): () => _moveLine(1),
        const SingleActivator(LogicalKeyboardKey.arrowUp, alt: true, shift: true): () => _duplicateLine(-1),
        const SingleActivator(LogicalKeyboardKey.arrowDown, alt: true, shift: true): () => _duplicateLine(1),
        const SingleActivator(LogicalKeyboardKey.keyK, control: true, shift: true): _deleteLine,
        const SingleActivator(LogicalKeyboardKey.keyK, meta: true, shift: true): _deleteLine,
      },
      child: Focus(
        autofocus: true,
        child: CodeTheme(
          data: CodeThemeData(styles: customTheme),
          child: GestureDetector(
            onTap: () {
              _focusNode.requestFocus();
              // Move cursor to the end of the text
              _controller.selection = TextSelection.collapsed(offset: _controller.text.length);
            },
            behavior: HitTestBehavior.opaque,
            child: Container(
              color: const Color(0xFF1E1E1E),
              child: Theme(
                data: Theme.of(context).copyWith(
                  inputDecorationTheme: const InputDecorationTheme(
                    border: InputBorder.none,
                    filled: false,
                  ),
                ),
                child: isWordWrap
                  ? CodeField(
                      key: ValueKey('code_field_wrap_true_$editorFontSize'),
                      wrap: true,
                      expands: false,
                      controller: _controller,
                      focusNode: _focusNode,
                      undoController: widget.tab.undoController,
                      maxLines: null,
                      textStyle: codeTextStyle,
                      lineNumberBuilder: lineNumberBuilder,
                      activeLine: _currentLine + 1,
                      activeLineColor: Colors.white.withOpacity(0.08),
                      gutterStyle: GutterStyle(
                        textStyle: gutterTextStyle,
                        textAlign: TextAlign.right,
                        background: const Color(0xFF1E1E1E),
                        margin: 16,
                        width: gutterWidth,
                        showErrors: false,
                        showFoldingHandles: true,
                      ),
                    )
                  : CodeField(
                      key: ValueKey('code_field_wrap_false_$editorFontSize'),
                      wrap: false,
                      expands: true,
                      controller: _controller,
                      focusNode: _focusNode,
                      undoController: widget.tab.undoController,
                      textStyle: codeTextStyle,
                      lineNumberBuilder: lineNumberBuilder,
                      activeLine: _currentLine + 1,
                      activeLineColor: Colors.white.withOpacity(0.08),
                      gutterStyle: GutterStyle(
                        textStyle: gutterTextStyle,
                        textAlign: TextAlign.right,
                        background: const Color(0xFF1E1E1E),
                        margin: 16,
                        width: gutterWidth,
                        showErrors: false,
                        showFoldingHandles: true,
                      ),
                    ),
              ),
            ),
          ),
        ),
      ),
    );
  }
}

/// Status bar widget — VS Code style blue bottom bar.
class EditorStatusBar extends StatelessWidget {
  final EditorTab? tab;
  final int line;
  final int column;

  const EditorStatusBar({super.key, this.tab, this.line = 1, this.column = 1});

  @override
  Widget build(BuildContext context) {
    final modified = tab?.isModified == true;
    return Container(
      height: 22,
      padding: const EdgeInsets.symmetric(horizontal: 10),
      decoration: const BoxDecoration(
        color: Color(0xFF2568E7),
      ),
      child: Row(
        children: [
          if (tab != null) ...[
            HugeIcon(icon: HugeIcons.strokeRoundedSourceCode, size: 12, color: Colors.white70),
            const SizedBox(width: 4),
            const Text(
              'RPL',
              style: TextStyle(
                fontSize: 11,
                color: Colors.white,
                fontFamily: 'monospace',
              ),
            ),
            const SizedBox(width: 10),
            const Text(
              'UTF-8',
              style: TextStyle(
                fontSize: 11,
                color: Colors.white70,
                fontFamily: 'monospace',
              ),
            ),
          ],
          const Spacer(),
          Text(
            'Ln $line, Col $column',
            style: const TextStyle(
              fontSize: 11,
              color: Colors.white,
              fontFamily: 'monospace',
            ),
          ),
          if (modified) ...[
            const SizedBox(width: 6),
            const Icon(Icons.circle, size: 6, color: Colors.white),
          ],
        ],
      ),
    );
  }
}

/// File Tab Bar — VS Code style horizontal scrollable tab bar.
class EditorTabBar extends StatefulWidget {
  final List<EditorTab> tabs;
  final int? activeIndex;
  final void Function(int index)? onTap;
  final void Function(int index)? onClose;

  const EditorTabBar({
    super.key,
    required this.tabs,
    this.activeIndex,
    this.onTap,
    this.onClose,
  });

  @override
  State<EditorTabBar> createState() => _EditorTabBarState();
}

class _EditorTabBarState extends State<EditorTabBar> {
  final ScrollController _scrollController = ScrollController();

  @override
  void dispose() {
    _scrollController.dispose();
    super.dispose();
  }

  @override
  void didUpdateWidget(EditorTabBar oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (widget.activeIndex != oldWidget.activeIndex && widget.activeIndex != null) {
      _scrollToActive();
    }
  }

  void _scrollToActive() {
    if (!mounted || widget.activeIndex == null || widget.tabs.isEmpty) return;

    WidgetsBinding.instance.addPostFrameCallback((_) {
      if (!_scrollController.hasClients) return;

      final index = widget.activeIndex!;
      double offset = 0.0;
      for (int i = 0; i < index; i++) {
        final textLen = widget.tabs[i].fileName.length;
        final tabWidth = 32.0 + 14.0 + 6.0 + (textLen * 7.5) + 22.0;
        offset += tabWidth;
      }

      final activeTextLen = widget.tabs[index].fileName.length;
      final activeTabWidth = 32.0 + 14.0 + 6.0 + (activeTextLen * 7.5) + 22.0;

      final viewportWidth = _scrollController.position.viewportDimension;
      final maxScroll = _scrollController.position.maxScrollExtent;

      final centeredOffset = offset - (viewportWidth / 2) + (activeTabWidth / 2);
      final targetOffset = centeredOffset.clamp(0.0, maxScroll);

      _scrollController.animateTo(
        targetOffset,
        duration: const Duration(milliseconds: 250),
        curve: Curves.easeInOut,
      );
    });
  }

  @override
  Widget build(BuildContext context) {
    return Container(
      height: 35,
      color: const Color(0xFF252526),
      child: ListView.builder(
        controller: _scrollController,
        scrollDirection: Axis.horizontal,
        itemCount: widget.tabs.length,
        itemBuilder: (context, i) {
          final isActive = widget.activeIndex == i;
          return GestureDetector(
            onTap: () => widget.onTap?.call(i),
            child: Container(
              height: 35,
              padding: const EdgeInsets.symmetric(horizontal: 12),
              decoration: BoxDecoration(
                color: isActive ? const Color(0xFF1E1E1E) : const Color(0xFF2D2D2D),
                border: Border(
                  right: const BorderSide(color: Color(0xFF252526), width: 1),
                  top: BorderSide(
                    color: isActive ? const Color(0xFF2568E7) : Colors.transparent,
                    width: isActive ? 2 : 0,
                  ),
                ),
              ),
              child: Row(
                mainAxisSize: MainAxisSize.min,
                children: [
                  FileIconHelper.getFileIcon(widget.tabs[i].fileName, size: 14),
                  const SizedBox(width: 8),
                  Text(
                    widget.tabs[i].fileName,
                    style: TextStyle(
                      fontSize: 12,
                      color: isActive ? Colors.white : const Color(0xFF969696),
                    ),
                  ),
                  const SizedBox(width: 6),
                  GestureDetector(
                    onTap: () => widget.onClose?.call(i),
                    child: MouseRegion(
                      cursor: SystemMouseCursors.click,
                      child: Padding(
                        padding: const EdgeInsets.all(2.0),
                        child: Icon(
                          widget.tabs[i].isModified ? Icons.circle : Icons.close,
                          size: widget.tabs[i].isModified ? 8 : 14,
                          color: widget.tabs[i].isModified
                              ? const Color(0xFFE8E8E8)
                              : (isActive ? const Color(0xFF969696) : Colors.transparent),
                        ),
                      ),
                    ),
                  ),
                ],
              ),
            ),
          );
        },
      ),
    );
  }
}

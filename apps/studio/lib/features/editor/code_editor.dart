import 'dart:io';
import 'dart:async';
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
import 'rpl_languages.dart';
import 'editor_tab.dart';
import '../../features/theme/theme_state.dart';
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
  final VoidCallback? onChanged;

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

    _controller.addListener(_onTextChanged);
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
    if (ext == 'rpl') {
      return rpl;
    }
    if (ext == 'js') {
      return javascript;
    }
    if (ext == 'css') {
      return css;
    }
    return xml; // HTML/XML fallback
  }

  void _onTextChanged() {
    if (_controller.text != _content) {
      setState(() {
        _content = _controller.text;
        widget.tab.content = _content;
        widget.tab.isModified = true;
      });
      widget.onChanged?.call();
      
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
      backgroundColor: const Color(0xFF1E1E1E), // VS Code exact editor background
    ) ?? const TextStyle(backgroundColor: Color(0xFF1E1E1E));

    final codeTextStyle = TextStyle(
      fontFamily: 'monospace',
      fontSize: editorFontSize,
      height: 1.6,
    );

    final gutterTextStyle = TextStyle(
      color: const Color(0xFF858585),
      fontSize: editorFontSize,
      fontFamily: 'monospace',
      height: 1.6,
    );

    return CallbackShortcuts(
      bindings: <ShortcutActivator, VoidCallback>{
        const SingleActivator(LogicalKeyboardKey.keyS, control: true): save,
        const SingleActivator(LogicalKeyboardKey.keyS, meta: true): save,
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
              padding: const EdgeInsets.only(top: 4.0),
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
                      gutterStyle: GutterStyle(
                        textStyle: gutterTextStyle,
                        background: const Color(0xFF1E1E1E),
                        margin: 0,
                        width: 60,
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
                      gutterStyle: GutterStyle(
                        textStyle: gutterTextStyle,
                        background: const Color(0xFF1E1E1E),
                        margin: 0,
                        width: 60,
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
                  Icon(
                    _getFileIcon(widget.tabs[i].fileName),
                    size: 14,
                    color: isActive ? _getFileIconColor(widget.tabs[i].fileName) : const Color(0xFF858585),
                  ),
                  const SizedBox(width: 6),
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

  IconData _getFileIcon(String fileName) {
    final ext = fileName.split('.').last.toLowerCase();
    switch (ext) {
      case 'rpl':
        return Icons.code;
      case 'html':
        return Icons.public;
      case 'css':
        return Icons.style;
      case 'js':
        return Icons.javascript;
      case 'json':
        return Icons.data_object;
      case 'md':
        return Icons.insert_drive_file;
      default:
        return Icons.insert_drive_file;
    }
  }

  Color _getFileIconColor(String fileName) {
    final ext = fileName.split('.').last.toLowerCase();
    switch (ext) {
      case 'rpl':
        return const Color(0xFF519ABA);
      case 'html':
        return const Color(0xFFE44D26);
      case 'css':
        return const Color(0xFF42A5F5);
      case 'js':
        return const Color(0xFFDCDCAA);
      case 'json':
        return const Color(0xFFDCDCAA);
      default:
        return const Color(0xFF519ABA);
    }
  }
}

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:shared_preferences/shared_preferences.dart';

const _kLowEndModeKey = 'setting_low_end_mode';
const _kAutoSaveKey = 'setting_auto_save';
const _kWordWrapKey = 'setting_word_wrap';
const _kEditorFontSizeKey = 'setting_editor_font_size';
const _kTerminalFontSizeKey = 'setting_terminal_font_size';
const _kEditorThemeKey = 'setting_editor_theme';
const _kTerminalHeightKey = 'setting_terminal_height';

final settingsProvider = NotifierProvider<SettingsNotifier, SettingsState>(() {
  return SettingsNotifier();
});

class SettingsState {
  final bool isLowEndMode;
  final bool isAutoSave;
  final bool isWordWrap;
  final double editorFontSize;
  final double terminalFontSize;
  final String editorTheme;
  final double terminalHeight;

  const SettingsState({
    this.isLowEndMode = false,
    this.isAutoSave = false,
    this.isWordWrap = false,
    this.editorFontSize = 13.0,
    this.terminalFontSize = 12.0,
    this.editorTheme = 'VS2015',
    this.terminalHeight = 170.0,
  });

  SettingsState copyWith({
    bool? isLowEndMode,
    bool? isAutoSave,
    bool? isWordWrap,
    double? editorFontSize,
    double? terminalFontSize,
    String? editorTheme,
    double? terminalHeight,
  }) {
    return SettingsState(
      isLowEndMode: isLowEndMode ?? this.isLowEndMode,
      isAutoSave: isAutoSave ?? this.isAutoSave,
      isWordWrap: isWordWrap ?? this.isWordWrap,
      editorFontSize: editorFontSize ?? this.editorFontSize,
      terminalFontSize: terminalFontSize ?? this.terminalFontSize,
      editorTheme: editorTheme ?? this.editorTheme,
      terminalHeight: terminalHeight ?? this.terminalHeight,
    );
  }
}

class SettingsNotifier extends Notifier<SettingsState> {
  @override
  SettingsState build() {
    _loadSettings();
    return const SettingsState();
  }

  Future<void> _loadSettings() async {
    final prefs = await SharedPreferences.getInstance();
    final isLowEndMode = prefs.getBool(_kLowEndModeKey) ?? false;
    final isAutoSave = prefs.getBool(_kAutoSaveKey) ?? false;
    final isWordWrap = prefs.getBool(_kWordWrapKey) ?? false;
    final editorFontSize = prefs.getDouble(_kEditorFontSizeKey) ?? 13.0;
    final terminalFontSize = prefs.getDouble(_kTerminalFontSizeKey) ?? 12.0;
    final editorTheme = prefs.getString(_kEditorThemeKey) ?? 'VS2015';
    final terminalHeight = prefs.getDouble(_kTerminalHeightKey) ?? 170.0;
    state = state.copyWith(
      isLowEndMode: isLowEndMode,
      isAutoSave: isAutoSave,
      isWordWrap: isWordWrap,
      editorFontSize: editorFontSize,
      terminalFontSize: terminalFontSize,
      editorTheme: editorTheme,
      terminalHeight: terminalHeight,
    );
  }

  Future<void> toggleLowEndMode(bool value) async {
    final prefs = await SharedPreferences.getInstance();
    await prefs.setBool(_kLowEndModeKey, value);
    state = state.copyWith(isLowEndMode: value);
  }

  Future<void> toggleAutoSave(bool value) async {
    final prefs = await SharedPreferences.getInstance();
    await prefs.setBool(_kAutoSaveKey, value);
    state = state.copyWith(isAutoSave: value);
  }

  Future<void> toggleWordWrap(bool value) async {
    final prefs = await SharedPreferences.getInstance();
    await prefs.setBool(_kWordWrapKey, value);
    state = state.copyWith(isWordWrap: value);
  }

  Future<void> setEditorFontSize(double value) async {
    final prefs = await SharedPreferences.getInstance();
    await prefs.setDouble(_kEditorFontSizeKey, value);
    state = state.copyWith(editorFontSize: value);
  }

  Future<void> setTerminalFontSize(double value) async {
    final prefs = await SharedPreferences.getInstance();
    await prefs.setDouble(_kTerminalFontSizeKey, value);
    state = state.copyWith(terminalFontSize: value);
  }

  Future<void> setEditorTheme(String value) async {
    final prefs = await SharedPreferences.getInstance();
    await prefs.setString(_kEditorThemeKey, value);
    state = state.copyWith(editorTheme: value);
  }

  Future<void> setTerminalHeight(double value) async {
    final prefs = await SharedPreferences.getInstance();
    await prefs.setDouble(_kTerminalHeightKey, value);
    state = state.copyWith(terminalHeight: value);
  }
}

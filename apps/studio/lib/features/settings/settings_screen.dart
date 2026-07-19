import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:hugeicons/hugeicons.dart';
import 'settings_provider.dart';

class SettingsScreen extends ConsumerWidget {
  const SettingsScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final settings = ref.watch(settingsProvider);

    return Scaffold(
      backgroundColor: const Color(0xFF1E1E1E),
      appBar: AppBar(
        backgroundColor: const Color(0xFF2D2D30),
        title: const Text(
          'Pengaturan',
          style: TextStyle(color: Color(0xFFFFFFFF), fontWeight: FontWeight.w600),
        ),
        elevation: 0,
        scrolledUnderElevation: 0,
        surfaceTintColor: Colors.transparent,
        shadowColor: Colors.transparent,
        shape: const Border(),
        leading: IconButton(
          icon: HugeIcon(
            icon: HugeIcons.strokeRoundedArrowLeft01,
            color: Colors.white,
            size: 22,
          ),
          onPressed: () => Navigator.of(context).pop(),
        ),
      ),
      body: ListView(
        padding: const EdgeInsets.all(16.0),
        children: [
          // ── Performa Section ──
          _sectionHeader(HugeIcons.strokeRoundedDashboardSpeed02, 'Performa'),
          const SizedBox(height: 8),
          _settingCard(
            icon: HugeIcons.strokeRoundedCpuCharge,
            title: 'Mode Ringan (Low-End Mode)',
            subtitle:
                'Direkomendasikan untuk perangkat berspesifikasi rendah. Fitur ini akan menghemat RAM secara drastis dengan cara:\n'
                '• Mematikan animasi & efek visual\n'
                '• Membatasi maksimal 2 tab Editor terbuka\n'
                '• Menunda pewarnaan sintaks, atau mematikannya untuk file lebih dari 800 baris\n'
                '• Membersihkan RAM Browser ( WebView ) otomatis saat tidak aktif\n'
                '• Membatasi pembacaan data Database & SQL Query maksimal 20 baris',
            value: settings.isLowEndMode,
            onChanged: (v) => ref.read(settingsProvider.notifier).toggleLowEndMode(v),
          ),

          const SizedBox(height: 24),

          // ── Editor Section ──
          _sectionHeader(HugeIcons.strokeRoundedSourceCodeSquare, 'Editor'),
          const SizedBox(height: 8),
          _settingCard(
            icon: HugeIcons.strokeRoundedFloppyDisk,
            title: 'Auto Save',
            subtitle: 'Menyimpan file secara otomatis setiap kali Anda mengetik di editor.',
            value: settings.isAutoSave,
            onChanged: (v) => ref.read(settingsProvider.notifier).toggleAutoSave(v),
          ),
          // const SizedBox(height: 8),
          // _settingCard(
          //   icon: HugeIcons.strokeRoundedTextWrap,
          //   title: 'Bungkus Teks (Word Wrap)',
          //   subtitle: 'Otomatis memotong teks agar sesuai dengan lebar layar sehingga Anda tidak perlu scroll ke kanan.',
          //   value: settings.isWordWrap,
          //   onChanged: (v) => ref.read(settingsProvider.notifier).toggleWordWrap(v),
          // ),
          const SizedBox(height: 8),
          _settingSlider(
            icon: HugeIcons.strokeRoundedTextFont,
            title: 'Ukuran Font',
            subtitle: 'Mengatur besarnya teks di dalam Code Editor.',
            value: settings.editorFontSize,
            min: 10,
            max: 30,
            onChanged: (v) => ref.read(settingsProvider.notifier).setEditorFontSize(v),
          ),
          const SizedBox(height: 8),
          _settingDropdown(
            icon: HugeIcons.strokeRoundedPaintBoard,
            title: 'Tema Editor',
            subtitle: 'Pilih tema pewarnaan sintaks (Syntax Highlighting) favorit Anda.',
            value: settings.editorTheme,
            items: const ['VS2015', 'Monokai', 'Monokai Sublime', 'Dracula', 'GitHub', 'Atom One Dark'],
            onChanged: (v) {
              if (v != null) ref.read(settingsProvider.notifier).setEditorTheme(v);
            },
          ),
        ],
      ),
    );
  }

  static Widget _sectionHeader(dynamic iconData, String label) {
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 4.0),
      child: Row(
        children: [
          HugeIcon(icon: iconData, color: const Color(0xFF2568E7), size: 18),
          const SizedBox(width: 8),
          Text(
            label,
            style: const TextStyle(
              color: Color(0xFF2568E7),
              fontSize: 14,
              fontWeight: FontWeight.bold,
              letterSpacing: 0.3,
            ),
          ),
        ],
      ),
    );
  }

  static Widget _settingCard({
    required dynamic icon,
    required String title,
    required String subtitle,
    required bool value,
    required ValueChanged<bool> onChanged,
  }) {
    return Card(
      color: const Color(0xFF2D2D30),
      elevation: 0,
      shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(12)),
      child: Padding(
        padding: const EdgeInsets.symmetric(vertical: 4),
        child: SwitchListTile(
          activeColor: const Color(0xFF2568E7),
          secondary: HugeIcon(icon: icon, color: Colors.white70, size: 22),
          title: Padding(
            padding: const EdgeInsets.only(bottom: 6),
            child: Text(title, style: const TextStyle(color: Colors.white, fontWeight: FontWeight.w600, fontSize: 14)),
          ),
          subtitle: Text(subtitle, style: const TextStyle(color: Colors.white54, fontSize: 12, height: 1.4)),
          value: value,
          onChanged: onChanged,
        ),
      ),
    );
  }

  static Widget _settingDropdown({
    required dynamic icon,
    required String title,
    required String subtitle,
    required String value,
    required List<String> items,
    required ValueChanged<String?> onChanged,
  }) {
    return Card(
      color: const Color(0xFF2D2D30),
      elevation: 0,
      shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(12)),
      child: Padding(
        padding: const EdgeInsets.symmetric(vertical: 4, horizontal: 16),
        child: Row(
          children: [
            HugeIcon(icon: icon, color: Colors.white70, size: 22),
            const SizedBox(width: 16),
            Expanded(
              child: Padding(
                padding: const EdgeInsets.symmetric(vertical: 12),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(title, style: const TextStyle(color: Colors.white, fontWeight: FontWeight.w600, fontSize: 14)),
                    const SizedBox(height: 6),
                    Text(subtitle, style: const TextStyle(color: Colors.white54, fontSize: 12, height: 1.4)),
                  ],
                ),
              ),
            ),
            const SizedBox(width: 16),
            DropdownButtonHideUnderline(
              child: DropdownButton<String>(
                value: value,
                dropdownColor: const Color(0xFF1E1E1E),
                style: const TextStyle(color: Colors.white, fontSize: 13),
                icon: const Icon(Icons.arrow_drop_down, color: Colors.white54),
                items: items.map((String item) {
                  return DropdownMenuItem<String>(
                    value: item,
                    child: Text(item),
                  );
                }).toList(),
                onChanged: onChanged,
              ),
            ),
          ],
        ),
      ),
    );
  }

  static Widget _settingSlider({
    required dynamic icon,
    required String title,
    required String subtitle,
    required double value,
    required double min,
    required double max,
    required ValueChanged<double> onChanged,
  }) {
    return Card(
      color: const Color(0xFF2D2D30),
      elevation: 0,
      shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(12)),
      child: Padding(
        padding: const EdgeInsets.symmetric(vertical: 4, horizontal: 16),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Padding(
              padding: const EdgeInsets.symmetric(vertical: 12),
              child: Row(
                children: [
                  HugeIcon(icon: icon, color: Colors.white70, size: 22),
                  const SizedBox(width: 16),
                  Expanded(
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Text(title, style: const TextStyle(color: Colors.white, fontWeight: FontWeight.w600, fontSize: 14)),
                        const SizedBox(height: 6),
                        Text(subtitle, style: const TextStyle(color: Colors.white54, fontSize: 12, height: 1.4)),
                      ],
                    ),
                  ),
                  const SizedBox(width: 16),
                  Text('${value.toInt()} px', style: const TextStyle(color: Colors.white, fontWeight: FontWeight.bold, fontSize: 14)),
                ],
              ),
            ),
            SliderTheme(
              data: SliderThemeData(
                activeTrackColor: const Color(0xFF2568E7),
                inactiveTrackColor: Colors.white12,
                thumbColor: Colors.white,
                overlayColor: const Color(0xFF2568E7).withAlpha(51),
                trackHeight: 4,
              ),
              child: Slider(
                value: value,
                min: min,
                max: max,
                divisions: (max - min).toInt(),
                onChanged: onChanged,
              ),
            ),
          ],
        ),
      ),
    );
  }
}

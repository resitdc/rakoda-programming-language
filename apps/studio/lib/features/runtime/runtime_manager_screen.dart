import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:hugeicons/hugeicons.dart';
import '../../shared/file_icon_helper.dart';
import 'runtime_service.dart';
import 'runtime_model.dart';
import 'runtime_downloader.dart';

class RuntimeManagerScreen extends ConsumerWidget {
  const RuntimeManagerScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final manifestAsync = ref.watch(runtimeManifestProvider);
    final systemInfo = SystemInfo.getCurrent();

    return Scaffold(
      backgroundColor: const Color(0xFF1E1E1E),
      appBar: AppBar(
        backgroundColor: const Color(0xFF2D2D30),
        title: const Text(
          'Pengelola Runtime',
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
          onPressed: () => Navigator.pop(context),
        ),
      ),
      body: manifestAsync.when(
        loading: () => const Center(
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              SizedBox(
                width: 40,
                height: 40,
                child: CircularProgressIndicator(
                  color: Color(0xFF2568E7),
                  strokeWidth: 3,
                ),
              ),
              SizedBox(height: 16),
              Text(
                'Memuat runtime...',
                style: TextStyle(color: Colors.white38, fontSize: 13),
              ),
            ],
          ),
        ),
        error: (err, stack) => Center(
          child: Padding(
            padding: const EdgeInsets.all(32),
            child: Column(
              mainAxisAlignment: MainAxisAlignment.center,
              children: [
                Container(
                  width: 64,
                  height: 64,
                  decoration: BoxDecoration(
                    color: Colors.redAccent.withOpacity(0.1),
                    borderRadius: BorderRadius.circular(16),
                  ),
                  child: const Center(
                    child: HugeIcon(
                      icon: HugeIcons.strokeRoundedAlert02,
                      color: Colors.redAccent,
                      size: 28,
                    ),
                  ),
                ),
                const SizedBox(height: 20),
                const Text(
                  'Gagal Memuat',
                  style: TextStyle(
                    color: Colors.white,
                    fontSize: 16,
                    fontWeight: FontWeight.w600,
                  ),
                ),
                const SizedBox(height: 8),
                Text(
                  'Pastikan koneksi internet Anda stabil\n$err',
                  textAlign: TextAlign.center,
                  style: const TextStyle(color: Colors.white38, fontSize: 13),
                ),
                const SizedBox(height: 24),
                TextButton.icon(
                  onPressed: () => ref.refresh(runtimeManifestProvider),
                  icon: const HugeIcon(
                    icon: HugeIcons.strokeRoundedRefresh,
                    color: Color(0xFF2568E7),
                    size: 18,
                  ),
                  label: const Text('Coba Lagi'),
                  style: TextButton.styleFrom(
                    foregroundColor: const Color(0xFF2568E7),
                    padding: const EdgeInsets.symmetric(horizontal: 24, vertical: 12),
                    shape: RoundedRectangleBorder(
                      borderRadius: BorderRadius.circular(12),
                      side: BorderSide(color: const Color(0xFF2568E7).withOpacity(0.3)),
                    ),
                  ),
                ),
              ],
            ),
          ),
        ),
        data: (manifest) {
          if (manifest.runtimes.isEmpty) {
            return const Center(
              child: Text(
                'Tidak ada runtime tersedia',
                style: TextStyle(color: Colors.white38),
              ),
            );
          }

          final List<Map<String, dynamic>> flattenedVersions = [];
          for (final runtimeEntry in manifest.runtimes.entries) {
            final runtimeKey = runtimeEntry.key;
            final versions = runtimeEntry.value.versions;

            final sortedVersions = versions.keys.toList()..sort((a, b) => b.compareTo(a));

            for (final versionStr in sortedVersions) {
              final versionData = versions[versionStr]!;
              bool isCompatible = false;
              RuntimeTarget? compatibleTarget;

              final osMap = versionData.osTargets[systemInfo.os];
              if (osMap != null) {
                compatibleTarget = osMap[systemInfo.arch];
                if (compatibleTarget != null) {
                  isCompatible = true;
                }
              }

              flattenedVersions.add({
                'runtimeKey': runtimeKey,
                'version': versionStr,
                'isCompatible': isCompatible,
                'target': compatibleTarget,
              });
            }
          }

          return ListView.builder(
            padding: const EdgeInsets.all(16),
            itemCount: flattenedVersions.length,
            itemBuilder: (context, index) {
              final item = flattenedVersions[index];
              return _RuntimeCard(item: item);
            },
          );
        },
      ),
    );
  }
}


// -- Runtime card widget --

class _RuntimeCard extends ConsumerWidget {
  final Map<String, dynamic> item;
  const _RuntimeCard({required this.item});

  // Get accent color based on runtime type
  Color _getAccentColor(String runtimeKey) {
    switch (runtimeKey.toLowerCase()) {
      case 'node':
        return const Color(0xFF68A063); // Node.js green
      case 'php':
        return const Color(0xFF777BB4); // PHP purple
      case 'python':
        return const Color(0xFF3776AB); // Python blue
      case 'ruby':
        return const Color(0xFFCC342D); // Ruby red
      case 'go':
        return const Color(0xFF00ADD8); // Go cyan
      case 'rust':
        return const Color(0xFFDEA584); // Rust orange
      case 'java':
        return const Color(0xFFED8B00); // Java orange
      default:
        return const Color(0xFF2568E7);
    }
  }

  String _getDescription(String runtimeKey) {
    switch (runtimeKey.toLowerCase()) {
      case 'node':
        return 'JavaScript runtime untuk membangun aplikasi web dan server';
      case 'php':
        return 'Bahasa pemrograman server-side untuk web development';
      case 'python':
        return 'Bahasa pemrograman serbaguna untuk AI, web, dan data science';
      case 'ruby':
        return 'Bahasa pemrograman dinamis untuk web development';
      case 'go':
        return 'Bahasa pemrograman efisien dari Google';
      case 'rust':
        return 'Bahasa pemrograman sistem yang aman dan cepat';
      case 'java':
        return 'Bahasa pemrograman populer untuk enterprise';
      default:
        return 'Runtime environment';
    }
  }

  String _formatSize(int bytes) {
    if (bytes <= 0) return '';
    if (bytes < 1024 * 1024) return '${(bytes / 1024).toStringAsFixed(0)} KB';
    return '${(bytes / (1024 * 1024)).toStringAsFixed(1)} MB';
  }

  String _getExtension(String runtimeKey) {
    switch (runtimeKey.toLowerCase()) {
      case 'node': return 'js';
      case 'python': return 'py';
      case 'ruby': return 'rb';
      case 'rust': return 'rs';
      default: return runtimeKey.toLowerCase();
    }
  }

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final runtimeKey = item['runtimeKey'] as String;
    final versionStr = item['version'] as String;
    final isCompatible = item['isCompatible'] as bool;
    final compatibleTarget = item['target'] as RuntimeTarget?;
    final downloaderId = '$runtimeKey@$versionStr';
    final accentColor = _getAccentColor(runtimeKey);

    return Padding(
      padding: const EdgeInsets.only(bottom: 12),
      child: Container(
        decoration: BoxDecoration(
          color: const Color(0xFF1A1A20),
          borderRadius: BorderRadius.circular(16),
          border: Border.all(
            color: Colors.white.withOpacity(0.06),
          ),
        ),
        child: Padding(
          padding: const EdgeInsets.all(16),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Row(
                children: [
                  // Icon container with accent glow
                  Container(
                    width: 48,
                    height: 48,
                    decoration: BoxDecoration(
                      color: accentColor.withOpacity(0.1),
                      borderRadius: BorderRadius.circular(12),
                      border: Border.all(
                        color: accentColor.withOpacity(0.15),
                      ),
                    ),
                    child: Center(
                      child: FileIconHelper.getFileIcon('dummy.${_getExtension(runtimeKey)}', size: 28),
                    ),
                  ),
                  const SizedBox(width: 14),

                  // Title + version
                  Expanded(
                    child: Column(
                      crossAxisAlignment: CrossAxisAlignment.start,
                      children: [
                        Row(
                          children: [
                            Text(
                              runtimeKey.toUpperCase(),
                              style: const TextStyle(
                                color: Colors.white,
                                fontSize: 16,
                                fontWeight: FontWeight.w700,
                                letterSpacing: 0.3,
                              ),
                            ),
                            const SizedBox(width: 8),
                            Container(
                              padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 2),
                              decoration: BoxDecoration(
                                color: accentColor.withOpacity(0.12),
                                borderRadius: BorderRadius.circular(6),
                              ),
                              child: Text(
                                'v$versionStr',
                                style: TextStyle(
                                  color: accentColor,
                                  fontSize: 11,
                                  fontWeight: FontWeight.w600,
                                ),
                              ),
                            ),
                          ],
                        ),
                        const SizedBox(height: 4),
                        Text(
                          _getDescription(runtimeKey),
                          style: const TextStyle(
                            color: Colors.white38,
                            fontSize: 12,
                          ),
                          maxLines: 1,
                          overflow: TextOverflow.ellipsis,
                        ),
                      ],
                    ),
                  ),
                ],
              ),

              const SizedBox(height: 14),

              // Divider
              Container(
                height: 1,
                color: Colors.white.withOpacity(0.04),
              ),

              const SizedBox(height: 14),

              // Bottom row: status/action
              if (!isCompatible)
                _IncompatibleBanner(systemInfo: SystemInfo.getCurrent())
              else
                Consumer(
                  builder: (context, ref, child) {
                    final downloader = ref.watch(runtimeDownloaderProvider(downloaderId));
                    return ListenableBuilder(
                      listenable: downloader,
                      builder: (context, child) {
                        final downloadStatus = downloader.status;

                        if (downloadStatus.state == DownloadState.installed) {
                          return _InstalledRow(
                            accentColor: accentColor,
                            onDelete: () => _showDeleteDialog(context, downloader, runtimeKey),
                          );
                        }

                        if (downloadStatus.state == DownloadState.downloading ||
                            downloadStatus.state == DownloadState.extracting) {
                          return _DownloadingRow(
                            downloadStatus: downloadStatus,
                            accentColor: accentColor,
                          );
                        }

                        if (downloadStatus.state == DownloadState.error) {
                          return _ErrorRow(
                            error: downloadStatus.error ?? 'Terjadi kesalahan',
                            onRetry: () => downloader.downloadAndInstall(compatibleTarget!),
                          );
                        }

                        // Idle
                        return _DownloadRow(
                          size: compatibleTarget != null ? _formatSize(compatibleTarget.size) : '',
                          onDownload: () => downloader.downloadAndInstall(compatibleTarget!),
                        );
                      },
                    );
                  },
                ),
            ],
          ),
        ),
      ),
    );
  }

  void _showDeleteDialog(BuildContext context, RuntimeDownloaderNotifier downloader, String runtimeKey) {
    showDialog(
      context: context,
      builder: (ctx) => AlertDialog(
        backgroundColor: const Color(0xFF1E1E24),
        shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(16)),
        title: const Text(
          'Hapus Runtime?',
          style: TextStyle(color: Colors.white, fontWeight: FontWeight.w600),
        ),
        content: Text(
          'Runtime ${runtimeKey.toUpperCase()} akan dihapus dari perangkat Anda. Anda dapat mengunduhnya kembali kapan saja.',
          style: const TextStyle(color: Colors.white54, fontSize: 14),
        ),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(ctx),
            child: const Text('Batal', style: TextStyle(color: Colors.white54)),
          ),
          TextButton(
            onPressed: () {
              downloader.deleteRuntime();
              Navigator.pop(ctx);
            },
            style: TextButton.styleFrom(
              foregroundColor: Colors.redAccent,
            ),
            child: const Text('Hapus', style: TextStyle(fontWeight: FontWeight.w600)),
          ),
        ],
      ),
    );
  }
}

// -- Sub-widgets --

class _InstalledRow extends StatelessWidget {
  final Color accentColor;
  final VoidCallback onDelete;
  const _InstalledRow({required this.accentColor, required this.onDelete});

  @override
  Widget build(BuildContext context) {
    return Row(
      children: [
        Container(
          padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 5),
          decoration: BoxDecoration(
            color: const Color(0xFF22C55E).withOpacity(0.1),
            borderRadius: BorderRadius.circular(8),
            border: Border.all(color: const Color(0xFF22C55E).withOpacity(0.2)),
          ),
          child: const Row(
            mainAxisSize: MainAxisSize.min,
            children: [
              Icon(Icons.check_circle_rounded, color: Color(0xFF22C55E), size: 15),
              SizedBox(width: 6),
              Text(
                'Terpasang',
                style: TextStyle(
                  color: Color(0xFF22C55E),
                  fontSize: 12,
                  fontWeight: FontWeight.w600,
                ),
              ),
            ],
          ),
        ),
        const Spacer(),
        SizedBox(
          height: 32,
          child: TextButton.icon(
            onPressed: onDelete,
            icon: const HugeIcon(
              icon: HugeIcons.strokeRoundedDelete02,
              color: Colors.white30,
              size: 15,
            ),
            label: const Text('Hapus'),
            style: TextButton.styleFrom(
              foregroundColor: Colors.white30,
              padding: const EdgeInsets.symmetric(horizontal: 10),
              textStyle: const TextStyle(fontSize: 12),
              shape: RoundedRectangleBorder(
                borderRadius: BorderRadius.circular(8),
              ),
            ),
          ),
        ),
      ],
    );
  }
}

class _DownloadingRow extends StatelessWidget {
  final RuntimeDownloadStatus downloadStatus;
  final Color accentColor;
  const _DownloadingRow({required this.downloadStatus, required this.accentColor});

  @override
  Widget build(BuildContext context) {
    final isExtracting = downloadStatus.state == DownloadState.extracting;
    final progress = downloadStatus.progress;

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Row(
          children: [
            SizedBox(
              width: 16,
              height: 16,
              child: CircularProgressIndicator(
                value: isExtracting ? null : null,
                color: accentColor,
                strokeWidth: 2,
              ),
            ),
            const SizedBox(width: 10),
            Text(
              isExtracting ? 'Mengekstrak...' : 'Mengunduh... ${(progress * 100).toStringAsFixed(0)}%',
              style: TextStyle(
                color: accentColor,
                fontSize: 12,
                fontWeight: FontWeight.w500,
              ),
            ),
          ],
        ),
        const SizedBox(height: 10),
        ClipRRect(
          borderRadius: BorderRadius.circular(4),
          child: LinearProgressIndicator(
            value: isExtracting ? null : progress,
            backgroundColor: Colors.white.withOpacity(0.06),
            valueColor: AlwaysStoppedAnimation<Color>(accentColor),
            minHeight: 4,
          ),
        ),
      ],
    );
  }
}

class _ErrorRow extends StatelessWidget {
  final String error;
  final VoidCallback onRetry;
  const _ErrorRow({required this.error, required this.onRetry});

  @override
  Widget build(BuildContext context) {
    return Row(
      children: [
        Expanded(
          child: Row(
            children: [
              const HugeIcon(
                icon: HugeIcons.strokeRoundedAlert02,
                color: Colors.redAccent,
                size: 16,
              ),
              const SizedBox(width: 8),
              Expanded(
                child: Text(
                  error,
                  style: const TextStyle(color: Colors.redAccent, fontSize: 12),
                  maxLines: 1,
                  overflow: TextOverflow.ellipsis,
                ),
              ),
            ],
          ),
        ),
        const SizedBox(width: 8),
        SizedBox(
          height: 32,
          child: TextButton(
            onPressed: onRetry,
            style: TextButton.styleFrom(
              foregroundColor: Colors.redAccent,
              padding: const EdgeInsets.symmetric(horizontal: 12),
              textStyle: const TextStyle(fontSize: 12, fontWeight: FontWeight.w600),
              shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(8)),
            ),
            child: const Text('Coba Lagi'),
          ),
        ),
      ],
    );
  }
}

class _DownloadRow extends StatelessWidget {
  final String size;
  final VoidCallback onDownload;
  const _DownloadRow({required this.size, required this.onDownload});

  @override
  Widget build(BuildContext context) {
    return Row(
      children: [
        if (size.isNotEmpty) ...[
          HugeIcon(
            icon: HugeIcons.strokeRoundedDownload04,
            color: Colors.white.withOpacity(0.25),
            size: 14,
          ),
          const SizedBox(width: 6),
          Text(
            size,
            style: TextStyle(
              color: Colors.white.withOpacity(0.3),
              fontSize: 12,
            ),
          ),
        ],
        const Spacer(),
        SizedBox(
          height: 36,
          child: ElevatedButton.icon(
            onPressed: onDownload,
            icon: const HugeIcon(
              icon: HugeIcons.strokeRoundedDownload04,
              color: Colors.white,
              size: 16,
            ),
            label: const Text('Unduh'),
            style: ElevatedButton.styleFrom(
              backgroundColor: const Color(0xFF2568E7),
              foregroundColor: Colors.white,
              elevation: 0,
              padding: const EdgeInsets.symmetric(horizontal: 20),
              textStyle: const TextStyle(fontSize: 13, fontWeight: FontWeight.w600),
              shape: RoundedRectangleBorder(
                borderRadius: BorderRadius.circular(10),
              ),
            ),
          ),
        ),
      ],
    );
  }
}

class _IncompatibleBanner extends StatelessWidget {
  final SystemInfo systemInfo;
  const _IncompatibleBanner({required this.systemInfo});

  @override
  Widget build(BuildContext context) {
    return Container(
      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
      decoration: BoxDecoration(
        color: Colors.amber.withOpacity(0.06),
        borderRadius: BorderRadius.circular(8),
        border: Border.all(color: Colors.amber.withOpacity(0.1)),
      ),
      child: Row(
        children: [
          const HugeIcon(
            icon: HugeIcons.strokeRoundedAlert02,
            color: Colors.amber,
            size: 16,
          ),
          const SizedBox(width: 8),
          Expanded(
            child: Text(
              'Tidak didukung di ${systemInfo.os} (${systemInfo.arch})',
              style: TextStyle(
                color: Colors.amber.withOpacity(0.7),
                fontSize: 12,
              ),
            ),
          ),
        ],
      ),
    );
  }
}

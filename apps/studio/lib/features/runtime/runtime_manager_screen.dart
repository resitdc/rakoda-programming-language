import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:hugeicons/hugeicons.dart';
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
          'Runtime Manager',
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
          child: CircularProgressIndicator(color: Color(0xFF2568E7)),
        ),
        error: (err, stack) => Center(
          child: Column(
            mainAxisAlignment: MainAxisAlignment.center,
            children: [
              const Icon(Icons.error_outline, color: Colors.redAccent, size: 48),
              const SizedBox(height: 16),
              Text(
                'Terjadi kesalahan:\n$err',
                textAlign: TextAlign.center,
                style: const TextStyle(color: Colors.white70),
              ),
              const SizedBox(height: 16),
              ElevatedButton(
                onPressed: () => ref.refresh(runtimeManifestProvider),
                style: ElevatedButton.styleFrom(backgroundColor: const Color(0xFF2568E7)),
                child: const Text('Coba Lagi', style: TextStyle(color: Colors.white)),
              ),
            ],
          ),
        ),
        data: (manifest) {
          if (manifest.runtimes.isEmpty) {
            return const Center(child: Text('Tidak ada runtime tersedia', style: TextStyle(color: Colors.white70)));
          }
          
          return ListView.builder(
            padding: const EdgeInsets.all(16),
            itemCount: manifest.runtimes.length,
            itemBuilder: (context, index) {
              final runtimeKey = manifest.runtimes.keys.elementAt(index);
              final runtimeData = manifest.runtimes[runtimeKey]!;
              final latestVersion = runtimeData.versions[runtimeData.latest];
              
              bool isCompatible = false;
              RuntimeTarget? compatibleTarget;
              
              if (latestVersion != null) {
                final osMap = latestVersion.osTargets[systemInfo.os];
                if (osMap != null) {
                  compatibleTarget = osMap[systemInfo.arch];
                  if (compatibleTarget != null) {
                    isCompatible = true;
                  }
                }
              }

              return Card(
                color: const Color(0xFF2D2D30),
                margin: const EdgeInsets.only(bottom: 16),
                child: Padding(
                  padding: const EdgeInsets.all(16.0),
                  child: Row(
                    children: [
                      Container(
                        width: 48,
                        height: 48,
                        decoration: BoxDecoration(
                          color: const Color(0xFF1E1E1E),
                          borderRadius: BorderRadius.circular(8),
                        ),
                        child: Center(
                          child: Text(
                            runtimeKey.substring(0, 1).toUpperCase(),
                            style: const TextStyle(
                              color: Colors.white,
                              fontSize: 24,
                              fontWeight: FontWeight.bold,
                            ),
                          ),
                        ),
                      ),
                      const SizedBox(width: 16),
                      Expanded(
                        child: Column(
                          crossAxisAlignment: CrossAxisAlignment.start,
                          children: [
                            Text(
                              runtimeKey.toUpperCase(),
                              style: const TextStyle(
                                color: Colors.white,
                                fontSize: 18,
                                fontWeight: FontWeight.bold,
                              ),
                            ),
                            const SizedBox(height: 4),
                            Text(
                              'Versi: ${runtimeData.latest}',
                              style: TextStyle(
                                color: Colors.white.withOpacity(0.7),
                                fontSize: 14,
                              ),
                            ),
                            if (!isCompatible)
                              Padding(
                                padding: const EdgeInsets.only(top: 4),
                                child: Text(
                                  'Tidak didukung di ${systemInfo.os} (${systemInfo.arch})',
                                  style: const TextStyle(
                                    color: Colors.redAccent,
                                    fontSize: 12,
                                  ),
                                ),
                              ),
                          ],
                        ),
                      ),
                      if (isCompatible)
                        Consumer(
                          builder: (context, ref, child) {
                            final downloader = ref.watch(runtimeDownloaderProvider(runtimeKey));

                            return ListenableBuilder(
                              listenable: downloader,
                              builder: (context, child) {
                                final downloadStatus = downloader.status;

                                if (downloadStatus.state == DownloadState.installed) {
                                  return Row(
                                    children: [
                                      const Icon(Icons.check_circle, color: Colors.green),
                                      const SizedBox(width: 8),
                                      Text(
                                        'Terpasang',
                                        style: TextStyle(
                                          color: Colors.green.shade400,
                                          fontWeight: FontWeight.bold,
                                        ),
                                      ),
                                    ],
                                  );
                                }

                                if (downloadStatus.state == DownloadState.downloading || downloadStatus.state == DownloadState.extracting) {
                                  return Column(
                                    crossAxisAlignment: CrossAxisAlignment.end,
                                    children: [
                                      SizedBox(
                                        width: 100,
                                        child: LinearProgressIndicator(
                                          value: downloadStatus.state == DownloadState.extracting ? null : downloadStatus.progress,
                                          backgroundColor: Colors.white12,
                                          valueColor: const AlwaysStoppedAnimation<Color>(Color(0xFF2568E7)),
                                        ),
                                      ),
                                      const SizedBox(height: 8),
                                      Text(
                                        downloadStatus.state == DownloadState.extracting
                                            ? 'Mengekstrak...'
                                            : '${(downloadStatus.progress * 100).toStringAsFixed(1)}%',
                                        style: TextStyle(
                                          color: Colors.white.withOpacity(0.7),
                                          fontSize: 12,
                                        ),
                                      ),
                                    ],
                                  );
                                }

                                return ElevatedButton(
                                  onPressed: () {
                                    downloader.downloadAndInstall(compatibleTarget!);
                                  },
                                  style: ElevatedButton.styleFrom(
                                    backgroundColor: const Color(0xFF2568E7),
                                    foregroundColor: Colors.white,
                                    shape: RoundedRectangleBorder(
                                      borderRadius: BorderRadius.circular(8),
                                    ),
                                  ),
                                  child: const Text('Unduh'),
                                );
                              },
                            );
                          },
                        ),
                    ],
                  ),
                ),
              );
            },
          );
        },
      ),
    );
  }
}

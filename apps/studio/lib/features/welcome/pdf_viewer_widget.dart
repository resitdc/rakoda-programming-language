import 'dart:io';
import 'package:flutter/material.dart';
import 'package:pdfx/pdfx.dart';

class PdfViewerWidget extends StatefulWidget {
  final String filePath;

  const PdfViewerWidget({super.key, required this.filePath});

  @override
  State<PdfViewerWidget> createState() => _PdfViewerWidgetState();
}

class _PdfViewerWidgetState extends State<PdfViewerWidget> {
  late PdfController _pdfController;

  @override
  void initState() {
    super.initState();
    _pdfController = PdfController(
      document: PdfDocument.openFile(widget.filePath),
    );
  }

  @override
  void dispose() {
    _pdfController.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    return Container(
      color: const Color(0xFF1E1E1E),
      child: Column(
        children: [
          // Toolbar
          Container(
            height: 40,
            color: const Color(0xFF252526),
            padding: const EdgeInsets.symmetric(horizontal: 16),
            child: Row(
              children: [
                PdfPageNumber(
                  controller: _pdfController,
                  builder: (context, loadingState, page, pagesCount) => Text(
                    'Halaman $page dari ${pagesCount ?? 0}',
                    style: const TextStyle(
                      color: Colors.white70,
                      fontSize: 13,
                      fontWeight: FontWeight.w500,
                    ),
                  ),
                ),
                const Spacer(),
                IconButton(
                  tooltip: 'Halaman Sebelumnya',
                  icon: const Icon(
                    Icons.chevron_left,
                    color: Colors.white70,
                    size: 20,
                  ),
                  onPressed: () {
                    try {
                      _pdfController.previousPage(
                        duration: const Duration(milliseconds: 250),
                        curve: Curves.easeInOut,
                      );
                    } catch (_) {}
                  },
                ),
                IconButton(
                  tooltip: 'Halaman Berikutnya',
                  icon: const Icon(
                    Icons.chevron_right,
                    color: Colors.white70,
                    size: 20,
                  ),
                  onPressed: () {
                    try {
                      _pdfController.nextPage(
                        duration: const Duration(milliseconds: 250),
                        curve: Curves.easeInOut,
                      );
                    } catch (_) {}
                  },
                ),
              ],
            ),
          ),
          // PDF View Area
          Expanded(
            child: PdfView(
              controller: _pdfController,
              scrollDirection: Axis.vertical,
              builders: PdfViewBuilders<DefaultBuilderOptions>(
                options: const DefaultBuilderOptions(),
                documentLoaderBuilder: (_) => const Center(
                  child: CircularProgressIndicator(color: Color(0xFF2568E7)),
                ),
                pageLoaderBuilder: (_) => const Center(
                  child: CircularProgressIndicator(color: Color(0xFF2568E7)),
                ),
              ),
            ),
          ),
        ],
      ),
    );
  }
}

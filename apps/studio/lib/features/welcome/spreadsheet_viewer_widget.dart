import 'dart:io';
import 'package:flutter/material.dart';
import 'package:csv/csv.dart';
import 'package:excel/excel.dart';
import 'package:pluto_grid/pluto_grid.dart';

class SpreadsheetViewerWidget extends StatefulWidget {
  final String filePath;

  const SpreadsheetViewerWidget({Key? key, required this.filePath}) : super(key: key);

  @override
  State<SpreadsheetViewerWidget> createState() => _SpreadsheetViewerWidgetState();
}

class _SpreadsheetViewerWidgetState extends State<SpreadsheetViewerWidget> {
  bool _isLoading = true;
  String? _error;
  
  final List<String> _sheetNames = [];
  final Map<String, List<PlutoColumn>> _columnsMap = {};
  final Map<String, List<PlutoRow>> _rowsMap = {};

  @override
  void initState() {
    super.initState();
    _loadFile();
  }

  Future<void> _loadFile() async {
    setState(() {
      _isLoading = true;
      _error = null;
    });
    try {
      final file = File(widget.filePath);
      final bytes = await file.readAsBytes();
      
      if (widget.filePath.toLowerCase().endsWith('.csv')) {
        final content = await file.readAsString();
        final rowsAsListOfValues = const CsvDecoder().convert(content);
        _sheetNames.add('CSV');
        _parseMatrix('CSV', rowsAsListOfValues);
      } else {
        // excel
        final excel = Excel.decodeBytes(bytes);
        if (excel.tables.isEmpty) {
          throw Exception("Tidak ada sheet yang ditemukan");
        }
        
        for (final sheetName in excel.tables.keys) {
          final table = excel.tables[sheetName];
          if (table == null) continue;
          
          final List<List<dynamic>> rowsAsListOfValues = [];
          for (final row in table.rows) {
            rowsAsListOfValues.add(row.map((e) => e?.value).toList());
          }
          
          _sheetNames.add(sheetName);
          _parseMatrix(sheetName, rowsAsListOfValues);
        }
      }
    } catch (e) {
      setState(() {
        _error = e.toString();
        _isLoading = false;
      });
      return;
    }
    
    setState(() {
      _isLoading = false;
    });
  }

  void _parseMatrix(String sheetName, List<List<dynamic>> data) {
    if (data.isEmpty) {
      _columnsMap[sheetName] = [];
      _rowsMap[sheetName] = [];
      return;
    }

    // Ambil kolom pertama sebagai header
    final headerRow = data.first;
    List<PlutoColumn> cols = [];
    for (int i = 0; i < headerRow.length; i++) {
      cols.add(
        PlutoColumn(
          title: headerRow[i]?.toString() ?? 'Column $i',
          field: 'col_$i',
          type: PlutoColumnType.text(),
          readOnly: true,
        ),
      );
    }

    // Sisa baris sebagai data
    List<PlutoRow> rows = [];
    for (int i = 1; i < data.length; i++) {
      final row = data[i];
      final Map<String, PlutoCell> cells = {};
      for (int c = 0; c < cols.length; c++) {
        final val = c < row.length ? row[c]?.toString() ?? '' : '';
        cells['col_$c'] = PlutoCell(value: val);
      }
      rows.add(PlutoRow(cells: cells));
    }

    _columnsMap[sheetName] = cols;
    _rowsMap[sheetName] = rows;
  }

  @override
  Widget build(BuildContext context) {
    if (_isLoading) {
      return Container(
        color: Colors.white,
        child: const Center(child: CircularProgressIndicator()),
      );
    }
    if (_error != null) {
      return Container(
        color: Colors.white,
        child: Center(
          child: Text(
            "Error loading file: $_error",
            style: const TextStyle(color: Colors.red),
          ),
        ),
      );
    }
    
    if (_sheetNames.isEmpty) {
      return Container(
        color: Colors.white,
        child: const Center(
          child: Text("Data kosong", style: TextStyle(color: Colors.black87)),
        ),
      );
    }

    return Container(
      color: Colors.white,
      child: DefaultTabController(
        length: _sheetNames.length,
        child: Column(
          children: [
            if (_sheetNames.length > 1)
              Container(
                color: Colors.grey[200],
                child: TabBar(
                  isScrollable: true,
                  labelColor: Colors.blue[800],
                  unselectedLabelColor: Colors.grey[700],
                  indicatorColor: Colors.blue[800],
                  tabs: _sheetNames.map((name) => Tab(text: name)).toList(),
                ),
              ),
            Expanded(
              child: TabBarView(
                physics: const NeverScrollableScrollPhysics(),
                children: _sheetNames.map((name) {
                  final cols = _columnsMap[name] ?? [];
                  final rows = _rowsMap[name] ?? [];
                  
                  if (cols.isEmpty) {
                    return const Center(
                      child: Text("Sheet kosong", style: TextStyle(color: Colors.black87)),
                    );
                  }
                  
                  return PlutoGrid(
                    columns: cols,
                    rows: rows,
                    configuration: const PlutoGridConfiguration(),
                  );
                }).toList(),
              ),
            ),
          ],
        ),
      ),
    );
  }
}

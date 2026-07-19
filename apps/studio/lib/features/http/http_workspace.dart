import 'dart:convert';
import 'package:flutter/material.dart';
import 'package:flutter/services.dart';
import 'package:hugeicons/hugeicons.dart';
import 'package:http/http.dart' as http;
import 'package:flutter_code_editor/flutter_code_editor.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import '../settings/settings_provider.dart';
import 'package:flutter_highlight/themes/vs2015.dart';
import 'package:flutter_highlight/themes/monokai.dart';
import 'package:flutter_highlight/themes/monokai-sublime.dart';
import 'package:flutter_highlight/themes/dracula.dart';
import 'package:flutter_highlight/themes/github.dart';
import 'package:flutter_highlight/themes/atom-one-dark.dart';
import 'package:highlight/languages/json.dart';
import 'package:highlight/languages/xml.dart';

class HttpWorkspace extends StatefulWidget {
  const HttpWorkspace({super.key});

  @override
  State<HttpWorkspace> createState() => _HttpWorkspaceState();
}

class _HttpWorkspaceState extends State<HttpWorkspace> {
  final TextEditingController _urlController = TextEditingController(text: 'https://jsonplaceholder.typicode.com/todos/1');
  String _method = 'GET';
  final List<String> _methods = ['GET', 'POST', 'PUT', 'PATCH', 'DELETE'];

  int _reqTabIndex = 0; // 0: Params, 1: Headers, 2: Body
  int _resTabIndex = 0; // 0: Body, 1: Headers

  final List<Map<String, dynamic>> _queryParams = [{'key': '', 'value': '', 'active': true}];
  final List<Map<String, dynamic>> _headers = [{'key': 'Accept', 'value': 'application/json', 'active': true}, {'key': '', 'value': '', 'active': true}];
  
  String _bodyType = 'None'; // None, JSON, Form-Data
  final TextEditingController _bodyJsonController = TextEditingController(text: '{\n  \n}');
  final List<Map<String, dynamic>> _bodyFormData = [{'key': '', 'value': '', 'active': true}];

  bool _isLoading = false;
  http.Response? _response;
  Duration? _responseTime;
  String? _errorMsg;
  CodeController? _responseCodeController;

  bool _isSyncing = false;
  List<Map<String, dynamic>> _pathVariables = [];

  @override
  void initState() {
    super.initState();
    _urlController.addListener(_onUrlChanged);
    _onUrlChanged();
  }

  void _onUrlChanged() {
    if (_isSyncing) return;
    _isSyncing = true;
    final text = _urlController.text;

    // 1. Path variables (look for :id)
    final pathRegex = RegExp(r':([a-zA-Z0-9_]+)');
    final matches = pathRegex.allMatches(text);
    final newPathVars = <Map<String, dynamic>>[];
    for (var m in matches) {
      final key = m.group(1)!;
      final existing = _pathVariables.firstWhere((p) => p['key'] == key, orElse: () => <String, dynamic>{'key': key, 'value': ''});
      newPathVars.add({'key': key, 'value': existing['value']});
    }

    // 2. Query Params
    try {
      final uri = Uri.parse(text);
      if (uri.hasQuery || text.contains('?')) {
        final newQueryParams = <Map<String, dynamic>>[];
        uri.queryParametersAll.forEach((k, vList) {
          for (var v in vList) {
            final existing = _queryParams.firstWhere((p) => p['key'] == k && p['value'] == v, orElse: () => <String, dynamic>{'active': true});
            newQueryParams.add({'key': k, 'value': v, 'active': existing['active'] ?? true});
          }
        });
        
        for (var p in _queryParams) {
          if (p['active'] == false && p['key'].toString().isNotEmpty) {
            newQueryParams.add(Map<String, dynamic>.from(p));
          }
        }
        _ensureEmptyRow(newQueryParams);

        setState(() {
          _pathVariables = newPathVars;
          _queryParams.clear();
          _queryParams.addAll(newQueryParams);
        });
      } else {
        setState(() {
          _pathVariables = newPathVars;
          final disabled = _queryParams.where((p) => p['active'] == false && p['key'].toString().isNotEmpty).toList();
          _queryParams.clear();
          _queryParams.addAll(disabled);
          _ensureEmptyRow(_queryParams);
        });
      }
    } catch (_) {
      setState(() {
        _pathVariables = newPathVars;
      });
    }
    _isSyncing = false;
  }

  void _syncUrlFromParams() {
    if (_isSyncing) return;
    _isSyncing = true;
    
    try {
      String urlStr = _urlController.text;
      int qIndex = urlStr.indexOf('?');
      String baseUrl = qIndex != -1 ? urlStr.substring(0, qIndex) : urlStr;
      
      final activeParams = _queryParams.where((p) => p['active'] == true && p['key'].toString().isNotEmpty).toList();
      
      if (activeParams.isNotEmpty) {
        final queryParts = activeParams.map((p) {
          final k = Uri.encodeQueryComponent(p['key'].toString());
          final v = Uri.encodeQueryComponent(p['value'].toString());
          return '$k=$v';
        }).join('&');
        urlStr = '$baseUrl?$queryParts';
      } else {
        urlStr = baseUrl;
      }
      
      if (_urlController.text != urlStr) {
         _urlController.text = urlStr;
      }
    } catch (_) {}
    
    _isSyncing = false;
  }

  @override
  void dispose() {
    _urlController.dispose();
    _bodyJsonController.dispose();
    _responseCodeController?.dispose();
    super.dispose();
  }

  Future<void> _sendRequest() async {
    FocusScope.of(context).unfocus();
    if (_urlController.text.trim().isEmpty) return;

    setState(() {
      _isLoading = true;
      _response = null;
      _responseTime = null;
      _errorMsg = null;
    });

    final stopwatch = Stopwatch()..start();
    try {
      // 1. Build URL
      String urlStr = _urlController.text.trim();
      
      // Substitute Path Variables
      for (var p in _pathVariables) {
        final k = p['key'].toString();
        final v = p['value'].toString();
        if (v.isNotEmpty) {
          urlStr = urlStr.replaceAll(':$k', Uri.encodeComponent(v));
        }
      }

      if (!urlStr.startsWith('http://') && !urlStr.startsWith('https://')) {
        urlStr = 'https://$urlStr';
      }

      final uri = Uri.parse(urlStr);

      // 2. Build Headers
      final Map<String, String> reqHeaders = {};
      for (var h in _headers) {
        if (h['active'] == true && h['key'].toString().isNotEmpty) {
          reqHeaders[h['key']] = h['value'];
        }
      }

      if (_bodyType == 'JSON' && !reqHeaders.keys.any((k) => k.toLowerCase() == 'content-type')) {
        reqHeaders['Content-Type'] = 'application/json';
      }

      // 3. Request execution
      http.Response res;
      final method = _method.toUpperCase();

      if (method == 'GET') {
        res = await http.get(uri, headers: reqHeaders);
      } else if (method == 'DELETE') {
        res = await http.delete(uri, headers: reqHeaders);
      } else {
        // POST, PUT, PATCH with Body
        Object? bodyData;
        if (_bodyType == 'JSON') {
          bodyData = _bodyJsonController.text;
        } else if (_bodyType == 'Form-Data') {
          final form = <String, String>{};
          for (var f in _bodyFormData) {
            if (f['active'] == true && f['key'].toString().isNotEmpty) {
              form[f['key']] = f['value'];
            }
          }
          bodyData = form;
        }

        if (method == 'POST') {
          res = await http.post(uri, headers: reqHeaders, body: bodyData);
        } else if (method == 'PUT') {
          res = await http.put(uri, headers: reqHeaders, body: bodyData);
        } else {
          res = await http.patch(uri, headers: reqHeaders, body: bodyData);
        }
      }

      stopwatch.stop();

      // Setup Response Viewer
      String bodyText = res.body;
      dynamic lang = xml;
      try {
        final decoded = jsonDecode(res.body);
        bodyText = const JsonEncoder.withIndent('  ').convert(decoded);
        lang = json;
      } catch (_) {}

      _responseCodeController?.dispose();
      _responseCodeController = CodeController(
        text: bodyText,
        language: lang,
      );

      setState(() {
        _response = res;
        _responseTime = stopwatch.elapsed;
        _isLoading = false;
        _resTabIndex = 0;
      });

    } catch (e) {
      stopwatch.stop();
      setState(() {
        _errorMsg = e.toString();
        _isLoading = false;
      });
    }
  }

  void _ensureEmptyRow(List<Map<String, dynamic>> list) {
    if (list.isEmpty || list.last['key'].toString().isNotEmpty || list.last['value'].toString().isNotEmpty) {
      list.add({'key': '', 'value': '', 'active': true});
    }
  }

  @override
  Widget build(BuildContext context) {
    return Container(
      color: const Color(0xFF1E1E1E),
      child: Column(
        children: [
          _buildAddressBar(),
          Expanded(
            child: Column(
              children: [
                Expanded(flex: 1, child: _buildRequestPanel()),
                Container(height: 1, color: const Color(0xFF333333)),
                Expanded(flex: 1, child: _buildResponsePanel()),
              ],
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildAddressBar() {
    return Container(
      padding: const EdgeInsets.all(12),
      decoration: const BoxDecoration(
        color: Color(0xFF252526),
        border: Border(bottom: BorderSide(color: Color(0xFF333333))),
      ),
      child: Row(
        children: [
          // Method Dropdown
          Container(
            height: 36,
            padding: const EdgeInsets.symmetric(horizontal: 12),
            decoration: BoxDecoration(
              color: const Color(0xFF333333),
              borderRadius: BorderRadius.circular(4),
            ),
            child: DropdownButtonHideUnderline(
              child: DropdownButton<String>(
                value: _method,
                dropdownColor: const Color(0xFF252526),
                style: TextStyle(
                  color: _method == 'GET' ? Colors.green 
                       : _method == 'POST' ? Colors.orange 
                       : _method == 'PUT' ? Colors.blue 
                       : _method == 'PATCH' ? Colors.purpleAccent 
                       : _method == 'DELETE' ? Colors.red 
                       : Colors.blue,
                  fontWeight: FontWeight.bold,
                  fontSize: 13,
                ),
                items: _methods.map((m) {
                  Color itemColor = Colors.white;
                  if (m == 'GET') itemColor = Colors.green;
                  else if (m == 'POST') itemColor = Colors.orange;
                  else if (m == 'PUT') itemColor = Colors.blue;
                  else if (m == 'PATCH') itemColor = Colors.purpleAccent;
                  else if (m == 'DELETE') itemColor = Colors.red;

                  return DropdownMenuItem(
                    value: m, 
                    child: Text(
                      m,
                      style: TextStyle(
                        color: itemColor,
                        fontWeight: FontWeight.bold,
                        fontSize: 13,
                      ),
                    ),
                  );
                }).toList(),
                onChanged: (val) {
                  if (val != null) setState(() => _method = val);
                },
              ),
            ),
          ),
          const SizedBox(width: 8),
          // URL Input
          Expanded(
            child: SizedBox(
              height: 38,
              child: TextField(
                controller: _urlController,
                style: const TextStyle(color: Colors.white, fontSize: 13),
                decoration: InputDecoration(
                  hintText: 'Tulis URL',
                  hintStyle: const TextStyle(color: Colors.white38),
                  contentPadding: const EdgeInsets.symmetric(horizontal: 12),
                  filled: true,
                  fillColor: const Color(0xFF1E1E1E),
                  border: OutlineInputBorder(
                    borderRadius: BorderRadius.circular(4),
                    borderSide: const BorderSide(color: Color(0xFF333333)),
                  ),
                  enabledBorder: OutlineInputBorder(
                    borderRadius: BorderRadius.circular(4),
                    borderSide: const BorderSide(color: Color(0xFF333333)),
                  ),
                  focusedBorder: OutlineInputBorder(
                    borderRadius: BorderRadius.circular(4),
                    borderSide: const BorderSide(color: Color(0xFF2568E7)),
                  ),
                ),
                onSubmitted: (_) => _sendRequest(),
              ),
            ),
          ),
          const SizedBox(width: 8),
          // Send Button
          SizedBox(
            height: 36,
            child: ElevatedButton.icon(
              onPressed: _isLoading ? null : _sendRequest,
              icon: _isLoading 
                ? const SizedBox(width: 14, height: 14, child: CircularProgressIndicator(strokeWidth: 2, color: Colors.white))
                : const HugeIcon(icon: HugeIcons.strokeRoundedSent, size: 16, color: Colors.white),
              label: Text(_isLoading ? 'Mengirim' : 'Kirim'),
              style: ElevatedButton.styleFrom(
                backgroundColor: const Color(0xFF2568E7),
                foregroundColor: Colors.white,
                elevation: 0,
                shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(4)),
                padding: const EdgeInsets.symmetric(horizontal: 16),
              ),
            ),
          ),
        ],
      ),
    );
  }

  Widget _buildRequestPanel() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        // Tabs
        Container(
          height: 40,
          color: const Color(0xFF252526),
          child: Row(
            children: [
              _buildTab('Params', 0, _reqTabIndex, (i) => setState(() => _reqTabIndex = i)),
              _buildTab('Headers', 1, _reqTabIndex, (i) => setState(() => _reqTabIndex = i)),
              _buildTab('Body', 2, _reqTabIndex, (i) => setState(() => _reqTabIndex = i)),
            ],
          ),
        ),
        // Content
        Expanded(
          child: _reqTabIndex == 0 
            ? _buildParamsTab()
            : _reqTabIndex == 1 
              ? _buildKeyValueEditor(_headers)
              : _buildBodyEditor(),
        ),
      ],
    );
  }

  Widget _buildParamsTab() {
    return ListView(
      padding: const EdgeInsets.all(12),
      children: [
        const Text('Query Params', style: TextStyle(color: Colors.white70, fontSize: 13, fontWeight: FontWeight.bold)),
        const SizedBox(height: 8),
        for (int i = 0; i < _queryParams.length; i++)
          _buildKeyValueRow(_queryParams, i, onItemChanged: _syncUrlFromParams),
        
        if (_pathVariables.isNotEmpty) ...[
          const SizedBox(height: 16),
          const Text('Path Variables', style: TextStyle(color: Colors.white70, fontSize: 13, fontWeight: FontWeight.bold)),
          const SizedBox(height: 8),
          for (int i = 0; i < _pathVariables.length; i++)
            _buildPathVariableRow(i),
        ]
      ],
    );
  }

  Widget _buildPathVariableRow(int index) {
    final item = _pathVariables[index];
    return Padding(
      padding: const EdgeInsets.only(bottom: 8),
      child: Row(
        children: [
          const SizedBox(width: 32),
          Expanded(
            child: Container(
              height: 32,
              alignment: Alignment.centerLeft,
              padding: const EdgeInsets.symmetric(horizontal: 8),
              decoration: BoxDecoration(
                color: const Color(0xFF252526),
                borderRadius: BorderRadius.circular(2),
                border: Border.all(color: const Color(0xFF333333)),
              ),
              child: Text(item['key'], style: const TextStyle(color: Colors.white54, fontSize: 13)),
            ),
          ),
          const SizedBox(width: 8),
          Expanded(
            child: _buildSmallTextField(
              key: ValueKey('${item.hashCode}_val'),
              initialValue: item['value'],
              hint: 'Value',
              onChanged: (val) {
                item['value'] = val;
                setState((){});
              },
            ),
          ),
          const SizedBox(width: 32),
        ],
      ),
    );
  }

  Widget _buildKeyValueEditor(List<Map<String, dynamic>> list, {VoidCallback? onItemChanged}) {
    return ListView.builder(
      padding: const EdgeInsets.all(12),
      itemCount: list.length,
      itemBuilder: (context, index) => _buildKeyValueRow(list, index, onItemChanged: onItemChanged),
    );
  }

  Widget _buildKeyValueRow(List<Map<String, dynamic>> list, int index, {VoidCallback? onItemChanged}) {
    final item = list[index];
    return Padding(
      padding: const EdgeInsets.only(bottom: 8),
      child: Row(
        children: [
          Checkbox(
            value: item['active'],
            activeColor: const Color(0xFF2568E7),
            side: const BorderSide(color: Colors.white30, width: 1.5),
            onChanged: (val) {
              setState(() {
                item['active'] = val;
              });
              onItemChanged?.call();
            },
          ),
          Expanded(
            child: _buildSmallTextField(
              key: ValueKey('${item.hashCode}_key'),
              initialValue: item['key'],
              hint: 'Key',
              onChanged: (val) {
                item['key'] = val;
                _ensureEmptyRow(list);
                setState((){});
                onItemChanged?.call();
              },
            ),
          ),
          const SizedBox(width: 8),
          Expanded(
            child: _buildSmallTextField(
              key: ValueKey('${item.hashCode}_val'),
              initialValue: item['value'],
              hint: 'Value',
              onChanged: (val) {
                item['value'] = val;
                _ensureEmptyRow(list);
                setState((){});
                onItemChanged?.call();
              },
            ),
          ),
          IconButton(
            icon: const HugeIcon(icon: HugeIcons.strokeRoundedDelete01, size: 16, color: Colors.white38),
            onPressed: list.length > 1 ? () {
              setState(() {
                list.removeAt(index);
              });
              onItemChanged?.call();
            } : null,
          )
        ],
      ),
    );
  }

  Widget _buildSmallTextField({Key? key, required String initialValue, required String hint, required ValueChanged<String> onChanged}) {
    return SizedBox(
      height: 32,
      child: TextFormField(
        key: key,
        initialValue: initialValue,
        style: const TextStyle(fontSize: 13, color: Colors.white),
        decoration: InputDecoration(
          hintText: hint,
          hintStyle: const TextStyle(color: Colors.white24, fontSize: 13),
          contentPadding: const EdgeInsets.symmetric(horizontal: 8, vertical: 0),
          filled: true,
          fillColor: const Color(0xFF252526),
          border: OutlineInputBorder(
            borderRadius: BorderRadius.circular(2),
            borderSide: const BorderSide(color: Color(0xFF333333)),
          ),
          enabledBorder: OutlineInputBorder(
            borderRadius: BorderRadius.circular(2),
            borderSide: const BorderSide(color: Color(0xFF333333)),
          ),
          focusedBorder: OutlineInputBorder(
            borderRadius: BorderRadius.circular(2),
            borderSide: const BorderSide(color: Color(0xFF2568E7)),
          ),
        ),
        onChanged: onChanged,
      ),
    );
  }

  Widget _buildBodyEditor() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Padding(
          padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
          child: Row(
            children: ['None', 'JSON', 'Form-Data'].map((type) {
              final isSelected = _bodyType == type;
              return InkWell(
                onTap: () {
                  setState(() => _bodyType = type);
                },
                borderRadius: BorderRadius.circular(4),
                child: Padding(
                  padding: const EdgeInsets.only(right: 16, top: 4, bottom: 4),
                  child: Row(
                    mainAxisSize: MainAxisSize.min,
                    children: [
                      Radio<String>(
                        value: type,
                        groupValue: _bodyType,
                        fillColor: MaterialStateProperty.resolveWith<Color>((Set<MaterialState> states) {
                          if (states.contains(MaterialState.selected)) {
                            return const Color(0xFF2568E7);
                          }
                          return Colors.white54;
                        }),
                        onChanged: (val) {
                          if (val != null) {
                            setState(() => _bodyType = val);
                          }
                        },
                      ),
                      Text(
                        type,
                        style: TextStyle(
                          color: isSelected ? Colors.white : Colors.white70,
                          fontSize: 13,
                          fontWeight: isSelected ? FontWeight.w500 : FontWeight.normal,
                        ),
                      ),
                    ],
                  ),
                ),
              );
            }).toList(),
          ),
        ),
        Expanded(
          child: _bodyType == 'None'
            ? const Center(child: Text('Request ini gapunya body', style: TextStyle(color: Colors.white38)))
            : _bodyType == 'JSON'
              ? Container(
                  margin: const EdgeInsets.all(12.0),
                  decoration: BoxDecoration(
                    color: const Color(0xFF252526),
                    borderRadius: BorderRadius.circular(4),
                    border: Border.all(color: const Color(0xFF333333)),
                  ),
                  child: TextField(
                    controller: _bodyJsonController,
                    maxLines: null,
                    expands: true,
                    textAlignVertical: TextAlignVertical.top,
                    style: const TextStyle(fontFamily: 'monospace', fontSize: 13, color: Colors.white),
                    decoration: const InputDecoration(
                      border: InputBorder.none,
                      contentPadding: EdgeInsets.all(12),
                      filled: true,
                      fillColor: Colors.transparent,
                      hintText: 'Tulis JSON disini',
                      hintStyle: TextStyle(color: Colors.white24),
                    ),
                  ),
                )
              : _buildKeyValueEditor(_bodyFormData),
        )
      ],
    );
  }

  Widget _buildResponsePanel() {
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        // Tabs & Status
        Container(
          height: 40,
          color: const Color(0xFF252526),
          child: Row(
            children: [
              _buildTab('Body', 0, _resTabIndex, (i) => setState(() => _resTabIndex = i)),
              _buildTab('Headers', 1, _resTabIndex, (i) => setState(() => _resTabIndex = i)),
              const Spacer(),
              if (_response != null) ...[
                _buildStatusChip('Status: ${_response!.statusCode}', _response!.statusCode >= 200 && _response!.statusCode < 300 ? Colors.green : Colors.red),
                const SizedBox(width: 8),
                _buildStatusChip('Time: ${_responseTime?.inMilliseconds} ms', Colors.orange),
                const SizedBox(width: 8),
                _buildStatusChip('Size: ${_response!.bodyBytes.length} B', Colors.blue),
                const SizedBox(width: 8),
                if (_resTabIndex == 0)
                  IconButton(
                    icon: const HugeIcon(icon: HugeIcons.strokeRoundedCopy01, size: 16, color: Colors.white70),
                    tooltip: 'Copy Response',
                    padding: EdgeInsets.zero,
                    constraints: const BoxConstraints(minWidth: 24, minHeight: 24),
                    splashRadius: 16,
                    onPressed: () {
                      if (_response?.body != null) {
                        Clipboard.setData(ClipboardData(text: _response!.body));
                        ScaffoldMessenger.of(context).showSnackBar(
                          const SnackBar(
                            content: Text('Response disalin ke clipboard', style: TextStyle(color: Colors.white, fontSize: 13)),
                            backgroundColor: Color(0xFF252526),
                            duration: Duration(seconds: 2),
                          ),
                        );
                      }
                    },
                  ),
                const SizedBox(width: 12),
              ],
            ],
          ),
        ),
        // Content
        Expanded(
          child: _isLoading 
            ? const Center(child: CircularProgressIndicator())
            : _errorMsg != null 
              ? Center(child: Text('Error: $_errorMsg', style: const TextStyle(color: Colors.red)))
              : _response == null 
                ? const Center(child: HugeIcon(icon: HugeIcons.strokeRoundedApi, size: 64, color: Colors.white12))
                : _resTabIndex == 0
                  ? _buildResponseBody()
                  : _buildResponseHeaders(),
        ),
      ],
    );
  }

  Widget _buildStatusChip(String text, Color color) {
    return Text(
      text,
      style: TextStyle(color: color, fontSize: 12, fontWeight: FontWeight.w600),
    );
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

  Widget _buildResponseBody() {
    if (_responseCodeController == null) return const SizedBox();

    return Consumer(
      builder: (context, ref, child) {
        final settings = ref.watch(settingsProvider);
        final baseTheme = _getTheme(settings.editorTheme);
        final editorFontSize = settings.editorFontSize;

        final customTheme = Map<String, TextStyle>.from(baseTheme);
        customTheme['root'] = customTheme['root']?.copyWith(
          backgroundColor: const Color(0xFF1E1E1E),
        ) ?? const TextStyle(backgroundColor: const Color(0xFF1E1E1E));

        return Container(
      color: const Color(0xFF1E1E1E),
      width: double.infinity,
      height: double.infinity,
      child: CodeTheme(
        data: CodeThemeData(styles: customTheme),
        child: Theme(
          data: Theme.of(context).copyWith(
            inputDecorationTheme: const InputDecorationTheme(
              border: InputBorder.none,
              filled: false,
            ),
          ),
              child: SingleChildScrollView(
                child: CodeField(
                  controller: _responseCodeController!,
                  readOnly: true,
                  textStyle: TextStyle(
                    fontFamily: 'monospace', 
                    fontSize: editorFontSize,
                    height: 1.6,
                  ),
                  gutterStyle: GutterStyle(
                    textStyle: TextStyle(
                      color: const Color(0xFF858585),
                      fontSize: editorFontSize,
                      fontFamily: 'monospace',
                      height: 1.6,
                    ),
                    background: const Color(0xFF1E1E1E),
                    margin: 0,
                    width: 60,
                  ),
                ),
              ),
            ),
          ),
        );
      },
    );
  }

  Widget _buildResponseHeaders() {
    final headers = _response!.headers;
    return ListView.builder(
      padding: const EdgeInsets.all(12),
      itemCount: headers.length,
      itemBuilder: (context, index) {
        final key = headers.keys.elementAt(index);
        final val = headers[key];
        return Padding(
          padding: const EdgeInsets.only(bottom: 6),
          child: Row(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Expanded(flex: 1, child: SelectableText(key, style: const TextStyle(fontWeight: FontWeight.bold, color: Colors.white70))),
              const Text(':  ', style: TextStyle(color: Colors.white38)),
              Expanded(flex: 3, child: SelectableText(val ?? '', style: const TextStyle(color: Colors.white))),
            ],
          ),
        );
      },
    );
  }

  Widget _buildTab(String title, int index, int currentIndex, ValueChanged<int> onTap) {
    final isActive = index == currentIndex;
    return InkWell(
      onTap: () => onTap(index),
      child: Container(
        padding: const EdgeInsets.symmetric(horizontal: 16),
        decoration: BoxDecoration(
          border: Border(
            bottom: BorderSide(
              color: isActive ? const Color(0xFF4EC9B0) : Colors.transparent,
              width: 2,
            ),
          ),
        ),
        child: Center(
          child: Text(
            title,
            style: TextStyle(
              color: isActive ? Colors.white : Colors.white54,
              fontWeight: isActive ? FontWeight.w500 : FontWeight.normal,
              fontSize: 13,
            ),
          ),
        ),
      ),
    );
  }
}

import 'dart:io';
import 'dart:convert';
import 'dart:async';
import 'dart:typed_data';
import 'package:sqflite_common_ffi/sqflite_ffi.dart';
import 'package:path/path.dart' as p;

import '../identity/identity_service.dart';

enum ClassroomStatus { disconnected, hosting, connected }

class ConnectedStudent {
  final String uuid;
  final String name;
  final String avatar;

  ConnectedStudent({required this.uuid, required this.name, required this.avatar});

  Map<String, dynamic> toJson() => {
    'uuid': uuid,
    'name': name,
    'avatar': avatar,
  };

  factory ConnectedStudent.fromJson(Map<String, dynamic> json) {
    return ConnectedStudent(
      uuid: json['uuid'] ?? '',
      name: json['name'] ?? 'Unknown',
      avatar: json['avatar'] ?? '🐼',
    );
  }
}

class ClassroomEvent {
  final String uuid;
  final String name;
  final String avatar;
  final String text;
  final String? fileName;
  final String? fileBase64;
  final bool isMine;
  final DateTime timestamp;
  final String eventType;
  final int? selectionStart;
  final int? selectionEnd;

  ClassroomEvent({
    required this.uuid,
    required this.name,
    required this.avatar,
    required this.text,
    this.fileName,
    this.fileBase64,
    required this.isMine,
    this.eventType = 'chat',
    this.selectionStart,
    this.selectionEnd,
    DateTime? timestamp,
  }) : timestamp = timestamp ?? DateTime.now();

  Map<String, dynamic> toJson() => {
    'uuid': uuid,
    'name': name,
    'avatar': avatar,
    'text': text,
    if (fileName != null) 'fileName': fileName,
    if (fileBase64 != null) 'fileBase64': fileBase64,
    'eventType': eventType,
    if (selectionStart != null) 'selectionStart': selectionStart,
    if (selectionEnd != null) 'selectionEnd': selectionEnd,
  };

  factory ClassroomEvent.fromJson(Map<String, dynamic> json, {required bool isMine}) {
    return ClassroomEvent(
      uuid: json['uuid'] ?? '',
      name: json['name'] ?? 'Unknown',
      avatar: json['avatar'] ?? '🐼',
      text: json['text'] ?? '',
      fileName: json['fileName'],
      fileBase64: json['fileBase64'],
      isMine: isMine,
      eventType: json['eventType'] ?? 'chat',
      selectionStart: json['selectionStart'],
      selectionEnd: json['selectionEnd'],
      timestamp: json['timestamp'] != null ? DateTime.fromMillisecondsSinceEpoch(json['timestamp']) : null,
    );
  }
}

class ClassroomService {
  static final ClassroomService _instance = ClassroomService._internal();
  factory ClassroomService() => _instance;
  ClassroomService._internal();

  ServerSocket? _serverSocket;
  Socket? _clientSocket; // Used by student to connect to host
  RawDatagramSocket? _udpSocket; // Used for custom room discovery
  
  final List<Socket> _hostClientSockets = []; // Used by host to store connected students
  final Map<Socket, String> _socketToUuid = {};
  Database? _db;

  ClassroomStatus status = ClassroomStatus.disconnected;
  String? roomCode;
  bool isLiveCodeSharingEnabled = true;
  bool isBroadcastingToHost = false;
  
  final List<ClassroomEvent> messages = [];

  Future<void> initDatabase(String projectPath) async {
    if (_db != null) return;
    
    final rplStudioDir = Directory(p.join(projectPath, '.rpl_studio'));
    if (!rplStudioDir.existsSync()) {
      rplStudioDir.createSync();
      if (Platform.isWindows) {
        try {
          Process.runSync('attrib', ['+h', rplStudioDir.path]);
        } catch (_) {}
      }
    }

    sqfliteFfiInit();
    final databaseFactory = databaseFactoryFfi;
    final dbPath = p.join(rplStudioDir.path, 'chat.db');
    
    _db = await databaseFactory.openDatabase(
      dbPath,
      options: OpenDatabaseOptions(
        version: 1,
        onCreate: (db, version) async {
          await db.execute('''
            CREATE TABLE messages (
              id INTEGER PRIMARY KEY AUTOINCREMENT,
              uuid TEXT,
              name TEXT,
              avatar TEXT,
              text TEXT,
              file_name TEXT,
              is_mine INTEGER,
              event_type TEXT,
              timestamp INTEGER
            )
          ''');
        },
      ),
    );

    final result = await _db!.query('messages', orderBy: 'timestamp ASC');
    final loadedMessages = result.map((row) {
      return ClassroomEvent(
        uuid: row['uuid'] as String,
        name: row['name'] as String,
        avatar: row['avatar'] as String,
        text: row['text'] as String,
        fileName: row['file_name'] as String?,
        isMine: (row['is_mine'] as int) == 1,
        eventType: row['event_type'] as String,
        timestamp: DateTime.fromMillisecondsSinceEpoch(row['timestamp'] as int),
      );
    }).toList();

    messages.clear();
    messages.addAll(loadedMessages);
  }

  void _insertMessageToDb(ClassroomEvent msg) {
    if (_db == null) return;
    _db!.insert('messages', {
      'uuid': msg.uuid,
      'name': msg.name,
      'avatar': msg.avatar,
      'text': msg.text,
      'file_name': msg.fileName,
      'is_mine': msg.isMine ? 1 : 0,
      'event_type': msg.eventType,
      'timestamp': msg.timestamp.millisecondsSinceEpoch,
    });
  }
  
  List<ConnectedStudent> connectedStudents = [];

  final _messageController = StreamController<ClassroomEvent>.broadcast();
  Stream<ClassroomEvent> get onMessage => _messageController.stream;

  final _statusController = StreamController<ClassroomStatus>.broadcast();
  Stream<ClassroomStatus> get onStatusChanged => _statusController.stream;

  bool get isHost => _serverSocket != null;

  void _updateStatus(ClassroomStatus s) {
    status = s;
    if (!_statusController.isClosed) {
      _statusController.add(s);
    }
  }

  // --- Utility Room Code ---
  Future<String?> _getLocalIp() async {
    final interfaces = await NetworkInterface.list();
    
    // Sort interfaces to prioritize standard Wi-Fi / Ethernet and deprioritize VirtualBox/VMware/WSL
    interfaces.sort((a, b) {
      final nameA = a.name.toLowerCase();
      final nameB = b.name.toLowerCase();
      int scoreA = _getInterfaceScore(nameA);
      int scoreB = _getInterfaceScore(nameB);
      return scoreA.compareTo(scoreB);
    });

    for (var interface in interfaces) {
      for (var addr in interface.addresses) {
        if (addr.type == InternetAddressType.IPv4 && !addr.isLoopback) {
          return addr.address;
        }
      }
    }
    return null;
  }

  int _getInterfaceScore(String name) {
    if (name.contains('virtual') || name.contains('vmware') || name.contains('vbox') || name.contains('wsl') || name.contains('ethernet adapter vethernet')) {
      return 100; // lower priority
    }
    if (name.contains('wi-fi') || name.contains('wifi') || name.contains('wlan') || name.contains('en0')) {
      return 0; // highest priority
    }
    if (name.contains('eth') || name.contains('ethernet') || name.contains('en')) {
      return 1; // high priority
    }
    return 50; // default
  }

  static const String _base32Chars = "0123456789ABCDEFGHJKMNPQRSTVWXYZ";

  String _encodeRoomCode(String ip, int port) {
    final parts = ip.split('.');
    final b = BytesBuilder();
    for (var p in parts) {
      b.addByte(int.parse(p));
    }
    b.addByte(port >> 8);
    b.addByte(port & 0xFF);
    
    final hexString = b.toBytes().map((e) => e.toRadixString(16).padLeft(2, '0')).join();
    var bigInt = BigInt.parse(hexString, radix: 16);
    
    if (bigInt == BigInt.zero) return "0";
    String result = "";
    final base = BigInt.from(32);
    while (bigInt > BigInt.zero) {
      final rem = (bigInt % base).toInt();
      result = _base32Chars[rem] + result;
      bigInt = bigInt ~/ base;
    }
    
    // Format dengan hyphen di tengah agar mudah dibaca, misal: ABCD-EFGH
    if (result.length > 5) {
      final mid = result.length ~/ 2;
      return '${result.substring(0, mid)}-${result.substring(mid)}';
    }
    return result;
  }

  List<dynamic>? _decodeRoomCode(String code) {
    try {
      final cleanCode = code.replaceAll('-', '').replaceAll(' ', '').toUpperCase().trim();
      
      // Auto-correct common typos from students (O->0, I/L->1, U->V)
      final correctedCode = cleanCode
          .replaceAll('O', '0')
          .replaceAll('I', '1')
          .replaceAll('L', '1')
          .replaceAll('U', 'V');

      BigInt bigInt = BigInt.zero;
      final base = BigInt.from(32);
      for (int i = 0; i < correctedCode.length; i++) {
        final char = correctedCode[i];
        final val = _base32Chars.indexOf(char);
        if (val == -1) return null; // Invalid character
        bigInt = bigInt * base + BigInt.from(val);
      }
      
      final hexString = bigInt.toRadixString(16).padLeft(12, '0');
      
      if (hexString.length != 12) return null;
      
      final ip1 = int.parse(hexString.substring(0, 2), radix: 16);
      final ip2 = int.parse(hexString.substring(2, 4), radix: 16);
      final ip3 = int.parse(hexString.substring(4, 6), radix: 16);
      final ip4 = int.parse(hexString.substring(6, 8), radix: 16);
      final port = int.parse(hexString.substring(8, 12), radix: 16);
      
      final ip = '$ip1.$ip2.$ip3.$ip4';
      return [ip, port];
    } catch (e) {
      return null;
    }
  }

  // --- Hosting ---
  Future<void> hostRoom({String? customCode}) async {
    try {
      final ip = await _getLocalIp();
      if (ip == null) throw Exception("Tidak ada koneksi WiFi/LAN.");
      
      _serverSocket = await ServerSocket.bind(ip, 0);
      
      if (customCode != null && customCode.trim().isNotEmpty) {
        roomCode = customCode.trim();
        try {
          _udpSocket = await RawDatagramSocket.bind(InternetAddress.anyIPv4, 45222, reuseAddress: true, reusePort: true);
          _udpSocket!.listen((RawSocketEvent e) {
            if (e == RawSocketEvent.read) {
              final datagram = _udpSocket!.receive();
              if (datagram != null) {
                final msg = utf8.decode(datagram.data);
                if (msg == 'FIND:$roomCode') {
                  final reply = utf8.encode('FOUND:${_serverSocket!.port}');
                  _udpSocket!.send(reply, datagram.address, datagram.port);
                }
              }
            }
          });
        } catch (_) {
          // Ignore if UDP binding fails, fallback to normal IP connect if they know it
        }
      } else {
        roomCode = _encodeRoomCode(ip, _serverSocket!.port);
      }
      messages.clear();
      _hostClientSockets.clear();
      _socketToUuid.clear();
      connectedStudents.clear();
      
      // Teacher is also in the list? Usually teacher is host.
      // Let's add teacher to the list of connected students for UI purposes, or leave it out depending on preference.
      // We'll leave it out, connectedStudents = just students.

      _updateStatus(ClassroomStatus.hosting);

      _serverSocket!.listen((Socket socket) {
        _hostClientSockets.add(socket);
        _listenToSocket(socket, isHostSide: true);
        if (status != ClassroomStatus.connected) {
           _updateStatus(ClassroomStatus.connected);
        }
      });
    } catch (e) {
      disconnect();
      rethrow;
    }
  }

  // --- Joining ---
  Future<void> joinRoom(String code) async {
    try {
      final decoded = _decodeRoomCode(code.trim());
      String ip;
      int port;
      
      if (decoded != null) {
        ip = decoded[0] as String;
        port = decoded[1] as int;
      } else {
        // Try UDP Discovery for custom room code
        final udpSocket = await RawDatagramSocket.bind(InternetAddress.anyIPv4, 0);
        udpSocket.broadcastEnabled = true;
        
        final completer = Completer<List<dynamic>>();
        
        udpSocket.listen((RawSocketEvent e) {
          if (e == RawSocketEvent.read) {
            final datagram = udpSocket.receive();
            if (datagram != null) {
              final msg = utf8.decode(datagram.data);
              if (msg.startsWith('FOUND:')) {
                final tcpPort = int.parse(msg.substring(6));
                if (!completer.isCompleted) {
                  completer.complete([datagram.address.address, tcpPort]);
                }
              }
            }
          }
        });
        
        // Broadcast discovery message
        final payload = utf8.encode('FIND:${code.trim()}');
        udpSocket.send(payload, InternetAddress("255.255.255.255"), 45222);
        
        for(int i=0; i<3; i++) {
           await Future.delayed(const Duration(milliseconds: 200));
           if (!completer.isCompleted) {
             udpSocket.send(payload, InternetAddress("255.255.255.255"), 45222);
           }
        }
        
        try {
          final result = await completer.future.timeout(const Duration(seconds: 2));
          ip = result[0] as String;
          port = result[1] as int;
        } catch (e) {
          udpSocket.close();
          throw Exception("Room '$code' tidak ditemukan di jaringan lokal Anda.");
        }
        udpSocket.close();
      }

      _clientSocket = await Socket.connect(ip, port, timeout: const Duration(seconds: 5));
      roomCode = code.trim();
      messages.clear();
      connectedStudents.clear();
      
      _listenToSocket(_clientSocket!, isHostSide: false);
      _updateStatus(ClassroomStatus.connected);

      // Send join event
      final joinMsg = ClassroomEvent(
        uuid: IdentityService.uuid,
        name: IdentityService.name ?? 'Unknown',
        avatar: IdentityService.avatar ?? '🐼',
        text: '',
        isMine: true,
        eventType: 'join',
      );
      _clientSocket!.write(jsonEncode(joinMsg.toJson()) + '\n');

    } catch (e) {
      disconnect();
      rethrow;
    }
  }

  void _listenToSocket(Socket socket, {required bool isHostSide}) {
    socket.listen(
      (List<int> data) {
        final text = utf8.decode(data).trim();
        if (text.isNotEmpty) {
          try {
            final lines = text.split('\n');
            for (var line in lines) {
              if (line.isEmpty) continue;
              final jsonMap = jsonDecode(line);
              
              if (jsonMap['eventType'] == 'join') {
                if (isHostSide) {
                  final student = ConnectedStudent.fromJson(jsonMap);
                  connectedStudents.add(student);
                  _socketToUuid[socket] = student.uuid;
                  _broadcastStudentsList();
                  _updateStatus(status); // notify UI
                }
                continue;
              }

              if (jsonMap['eventType'] == 'students_list') {
                if (!isHostSide) {
                  final list = jsonMap['students'] as List;
                  connectedStudents = list.map((e) => ConnectedStudent.fromJson(e)).toList();
                  _updateStatus(status); // notify UI
                }
                continue;
              }

              final msg = ClassroomEvent.fromJson(jsonMap, isMine: false);
              
              if (msg.eventType == 'chat') {
                messages.add(msg);
                _insertMessageToDb(msg);
                if (isHostSide) {
                  // Relay chat to other students
                  _broadcast(line + '\n', exclude: socket);
                }
              }
              _messageController.add(msg);
            }
          } catch (e) {
            // Ignore parse errors
          }
        }
      },
      onDone: () => _handleSocketClosed(socket, isHostSide),
      onError: (_) => _handleSocketClosed(socket, isHostSide),
    );
  }

  void _handleSocketClosed(Socket socket, bool isHostSide) {
    if (isHostSide) {
      _hostClientSockets.remove(socket);
      final uuid = _socketToUuid.remove(socket);
      if (uuid != null) {
        connectedStudents.removeWhere((s) => s.uuid == uuid);
        _broadcastStudentsList();
        _updateStatus(status);
      }
      socket.close();
    } else {
      disconnect();
    }
  }

  void _broadcastStudentsList() {
    final data = {
      'eventType': 'students_list',
      'students': connectedStudents.map((s) => s.toJson()).toList(),
    };
    _broadcast(jsonEncode(data) + '\n');
  }

  void _broadcast(String data, {Socket? exclude}) {
    for (var s in _hostClientSockets) {
      if (s != exclude) {
        try {
          s.write(data);
        } catch (_) {}
      }
    }
  }

  void sendMessage(String text, {String? fileName, String? fileBase64}) {
    if (status != ClassroomStatus.connected) return;
    if (!isHost && _clientSocket == null) return;
    
    final msg = ClassroomEvent(
      uuid: IdentityService.uuid,
      name: IdentityService.name ?? 'Unknown',
      avatar: IdentityService.avatar ?? '🐼',
      text: text,
      fileName: fileName,
      fileBase64: fileBase64,
      isMine: true,
      eventType: 'chat',
    );
    
    final jsonString = jsonEncode(msg.toJson()) + '\n';
    
    if (isHost) {
      _broadcast(jsonString);
    } else {
      _clientSocket!.write(jsonString);
    }
    
    messages.add(msg);
    _insertMessageToDb(msg);
    _messageController.add(msg);
  }

  void setLiveCodeSharing(bool enabled) {
    isLiveCodeSharingEnabled = enabled;
    if (!enabled && status == ClassroomStatus.connected && isHost) {
      final msg = ClassroomEvent(
        uuid: IdentityService.uuid,
        name: IdentityService.name ?? 'Unknown',
        avatar: IdentityService.avatar ?? '🐼',
        text: '',
        isMine: true,
        eventType: 'hide_live_code',
      );
      _broadcast(jsonEncode(msg.toJson()) + '\n');
    }
  }

  void broadcastLiveCode(String codeContent, {int? selectionStart, int? selectionEnd}) {
    if (status != ClassroomStatus.connected || !isHost || !isLiveCodeSharingEnabled) return;
    
    final msg = ClassroomEvent(
      uuid: IdentityService.uuid,
      name: IdentityService.name ?? 'Unknown',
      avatar: IdentityService.avatar ?? '🐼',
      text: codeContent,
      isMine: true,
      eventType: 'live_code',
      selectionStart: selectionStart,
      selectionEnd: selectionEnd,
    );
    
    _broadcast(jsonEncode(msg.toJson()) + '\n');
  }

  void requestStudentCode(String targetUuid, bool isRequesting) {
    if (!isHost) return;
    
    final msg = ClassroomEvent(
      uuid: targetUuid, // Menggunakan field uuid sebagai target
      name: '',
      avatar: '',
      text: isRequesting.toString(),
      isMine: true,
      eventType: 'request_code_stream',
    );
    
    _broadcast(jsonEncode(msg.toJson()) + '\n');
  }

  void sendStudentLiveCode(String codeContent, {int? selectionStart, int? selectionEnd}) {
    if (status != ClassroomStatus.connected || isHost || !isBroadcastingToHost) return;
    
    final msg = ClassroomEvent(
      uuid: IdentityService.uuid,
      name: IdentityService.name ?? 'Unknown',
      avatar: IdentityService.avatar ?? '🐼',
      text: codeContent,
      isMine: true,
      eventType: 'student_live_code',
      selectionStart: selectionStart,
      selectionEnd: selectionEnd,
    );
    
    _clientSocket?.write(jsonEncode(msg.toJson()) + '\n');
  }

  void broadcastTerminalOutput(String output) {
    if (status != ClassroomStatus.connected || !isHost || !isLiveCodeSharingEnabled) return;
    
    final msg = ClassroomEvent(
      uuid: IdentityService.uuid,
      name: IdentityService.name ?? 'Unknown',
      avatar: IdentityService.avatar ?? '🐼',
      text: output,
      isMine: true,
      eventType: 'terminal_output',
    );
    
    _broadcast(jsonEncode(msg.toJson()) + '\n');
  }

  void disconnect() {
    _clientSocket?.close();
    _clientSocket = null;
    
    _udpSocket?.close();
    _udpSocket = null;
    
    for (var s in _hostClientSockets) {
      s.close();
    }
    _hostClientSockets.clear();
    _socketToUuid.clear();
    
    _serverSocket?.close();
    _serverSocket = null;
    
    roomCode = null;
    connectedStudents.clear();
    _updateStatus(ClassroomStatus.disconnected);
  }

  void dispose() {
    disconnect();
  }
}

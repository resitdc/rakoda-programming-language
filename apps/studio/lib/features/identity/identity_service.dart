import 'package:shared_preferences/shared_preferences.dart';
import 'package:uuid/uuid.dart';

class IdentityService {
  static const String _keyUuid = 'identity_uuid';
  static const String _keyName = 'identity_name';
  static const String _keyAvatar = 'identity_avatar';

  static String? _uuid;
  static String? _name;
  static String? _avatar;

  static Future<void> init() async {
    final prefs = await SharedPreferences.getInstance();
    
    _uuid = prefs.getString(_keyUuid);
    _name = prefs.getString(_keyName);
    _avatar = prefs.getString(_keyAvatar);

    if (_uuid == null) {
      _uuid = const Uuid().v4();
      await prefs.setString(_keyUuid, _uuid!);
    }
  }

  static String get uuid => _uuid ?? '';
  static String? get name => _name;
  static String? get avatar => _avatar;

  static bool get hasIdentity => _name != null && _name!.isNotEmpty;

  static Future<void> saveIdentity(String name, String avatar) async {
    final prefs = await SharedPreferences.getInstance();
    await prefs.setString(_keyName, name);
    await prefs.setString(_keyAvatar, avatar);
    _name = name;
    _avatar = avatar;
  }
}

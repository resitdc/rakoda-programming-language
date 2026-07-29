class RuntimeManifest {
  final Map<String, RuntimeData> runtimes;

  RuntimeManifest({required this.runtimes});

  factory RuntimeManifest.fromJson(Map<String, dynamic> json) {
    Map<String, RuntimeData> runtimesMap = {};
    json.forEach((key, value) {
      runtimesMap[key] = RuntimeData.fromJson(value);
    });
    return RuntimeManifest(runtimes: runtimesMap);
  }
}

class RuntimeData {
  final String latest;
  final Map<String, RuntimeVersion> versions;

  RuntimeData({required this.latest, required this.versions});

  factory RuntimeData.fromJson(Map<String, dynamic> json) {
    Map<String, RuntimeVersion> versionsMap = {};
    if (json['versions'] != null) {
      (json['versions'] as Map<String, dynamic>).forEach((key, value) {
        versionsMap[key] = RuntimeVersion.fromJson(value);
      });
    }
    return RuntimeData(latest: json['latest'] ?? '', versions: versionsMap);
  }
}

class RuntimeVersion {
  final Map<String, Map<String, RuntimeTarget>> osTargets;

  RuntimeVersion({required this.osTargets});

  factory RuntimeVersion.fromJson(Map<String, dynamic> json) {
    Map<String, Map<String, RuntimeTarget>> osTargetsMap = {};
    json.forEach((osKey, osValue) {
      Map<String, RuntimeTarget> archMap = {};
      (osValue as Map<String, dynamic>).forEach((archKey, archValue) {
        archMap[archKey] = RuntimeTarget.fromJson(archValue);
      });
      osTargetsMap[osKey] = archMap;
    });
    return RuntimeVersion(osTargets: osTargetsMap);
  }
}

class RuntimeTarget {
  final String url;
  final String sha256;
  final int size;

  RuntimeTarget({required this.url, required this.sha256, required this.size});

  factory RuntimeTarget.fromJson(Map<String, dynamic> json) {
    return RuntimeTarget(
      url: json['url'] ?? '',
      sha256: json['sha256'] ?? '',
      size: json['size'] ?? 0,
    );
  }
}

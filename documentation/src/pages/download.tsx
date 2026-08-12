import React, { useState, useEffect } from 'react';
import Layout from '@theme/Layout';
import Heading from '@theme/Heading';
import styles from './download.module.css';

// ─── Icons ───────────────────────────────────────────────

const WindowsIcon = () => (
  <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
    <path d="M0 3.449L9.75 2.1v9.451H0m10.949-9.602L24 0v11.4H10.949M0 12.6h9.75v9.451L0 20.699M10.949 12.6H24V24l-12.9-1.801"/>
  </svg>
);

const MacIcon = () => (
  <svg width="20" height="20" viewBox="0 0 24 24" fill="currentColor">
    <path d="M12.152 6.896c-.948 0-2.415-1.078-3.96-1.04-2.04.027-3.91 1.183-4.961 3.014-2.117 3.675-.546 9.103 1.519 12.09 1.013 1.454 2.208 3.09 3.792 3.039 1.52-.065 2.09-.987 3.935-.987 1.831 0 2.35.987 3.96.948 1.637-.026 2.676-1.48 3.676-2.948 1.156-1.688 1.636-3.325 1.662-3.415-.039-.013-3.182-1.221-3.22-4.857-.026-3.04 2.48-4.494 2.597-4.559-1.429-2.09-3.623-2.324-4.39-2.376-2-.156-3.675 1.09-4.61 1.09zM15.53 3.83c.843-1.012 1.4-2.427 1.245-3.83-1.207.052-2.662.805-3.532 1.818-.78.896-1.454 2.338-1.273 3.714 1.338.104 2.715-.688 3.56-1.701z"/>
  </svg>
);

const LinuxIcon = () => (
  <svg fill="currentColor" width="20" height="20" viewBox="0 0 32 32">
    <path d="M14.923 8.080c-0.025 0.072-0.141 0.061-0.207 0.082-0.059 0.031-0.107 0.085-0.175 0.085-0.062 0-0.162-0.025-0.17-0.085-0.012-0.082 0.11-0.166 0.187-0.166 0.050-0.024 0.108-0.037 0.169-0.037 0.056 0 0.109 0.011 0.157 0.032l-0.003-0.001c0.022 0.009 0.038 0.030 0.038 0.055 0 0.003-0 0.005-0.001 0.008v0.025h0.004zM15.611 8.080v-0.027c-0.008-0.025 0.016-0.052 0.036-0.062 0.046-0.020 0.1-0.032 0.157-0.032 0.061 0 0.119 0.014 0.17 0.038l-0.002-0.001c0.079 0 0.2 0.084 0.187 0.169-0.007 0.061-0.106 0.082-0.169 0.082-0.069 0-0.115-0.054-0.176-0.085-0.065-0.023-0.182-0.010-0.204-0.081zM16.963 10.058c-0.532 0.337-1.161 0.574-1.835 0.666l-0.024 0.003c-0.606-0.035-1.157-0.248-1.607-0.588l0.007 0.005c-0.192-0.167-0.35-0.335-0.466-0.419-0.205-0.167-0.18-0.416-0.092-0.416 0.136 0.020 0.161 0.167 0.249 0.25 0.12 0.082 0.269 0.25 0.45 0.416 0.397 0.328 0.899 0.541 1.45 0.583l0.009 0.001c0.654-0.057 1.249-0.267 1.763-0.592l-0.016 0.010c0.244-0.169 0.556-0.417 0.81-0.584 0.195-0.17 0.186-0.334 0.349-0.334 0.16 0.020 0.043 0.167-0.184 0.415-0.246 0.188-0.527 0.381-0.818 0.56l-0.044 0.025z"/>
    <path d="M16.63 1.004c-0.194 0-0.394 0.010-0.6 0.026-5.281 0.416-3.88 6.007-3.961 7.87-0.050 1.426-0.534 2.729-1.325 3.792l0.013-0.018c-1.407 1.602-2.555 3.474-3.351 5.523l-0.043 0.127c-0.258 0.685-0.408 1.476-0.408 2.302 0 0.285 0.018 0.566 0.052 0.841l-0.003-0.033c-0.056 0.046-0.103 0.102-0.136 0.166l-0.001 0.003c-0.325 0.335-0.562 0.75-0.829 1.048-0.283 0.217-0.615 0.388-0.975 0.494l-0.021 0.005c-0.464 0.139-0.842 0.442-1.075 0.841l-0.005 0.009c-0.104 0.212-0.165 0.461-0.165 0.725 0 0.01 0 0.019 0 0.029v0c0.002 0.238 0.026 0.469 0.073 0.693l-0.004-0.023c0.056 0.219 0.088 0.471 0.088 0.73 0 0.17-0.014 0.337-0.041 0.5l0.002-0.018c-0.167 0.313-0.264 0.685-0.264 1.080 0 0.278 0.048 0.544 0.137 0.791l-0.005-0.016c0.273 0.388 0.686 0.662 1.164 0.749l0.011 0.002c1.274 0.107 2.451 0.373 3.561 0.78l-0.094-0.030c0.698 0.415 1.539 0.66 2.436 0.66 0.294 0 0.582-0.026 0.862-0.077l-0.029 0.004c0.667-0.151 1.211-0.586 1.504-1.169l0.006-0.013c0.734-0.004 1.537-0.336 2.824-0.417 0.873-0.072 1.967 0.334 3.22 0.25 0.037 0.159 0.086 0.298 0.148 0.429l-0.006-0.013 0.004 0.004c0.384 0.804 1.19 1.35 2.124 1.35 0.081 0 0.161-0.004 0.24-0.012l-0.010 0.001c1.151-0.17 2.139-0.768 2.813-1.623l0.007-0.009c0.843-0.768 1.827-1.401 2.905-1.853l0.067-0.025c0.432-0.191 0.742-0.585 0.81-1.059l0.001-0.007c-0.059-0.694-0.392-1.299-0.888-1.716l-0.004-0.003v-0.121l-0.004-0.004c-0.214-0.33-0.364-0.722-0.421-1.142l-0.002-0.015c-0.053-0.513-0.278-0.966-0.615-1.307v0h-0.004c-0.074-0.067-0.154-0.084-0.235-0.169-0.066-0.047-0.148-0.076-0.237-0.080h-0.001c0.195-0.602 0.308-1.294 0.308-2.013 0-0.94-0.193-1.835-0.541-2.647l0.017 0.044c-0.704-1.672-1.619-3.111-2.732-4.369l0.014 0.017c-1.105-1.082-1.828-2.551-1.948-4.187l-0.001-0.021c0.033-2.689 0.295-7.664-4.429-7.671z"/>
  </svg>
);

const AndroidIcon = () => (
  <svg fill="currentColor" height="20" width="20" viewBox="0 0 299.679 299.679">
    <path d="M181.122,299.679c10.02,0,18.758-8.738,18.758-18.758v-43.808h12.525c7.516,0,12.525-5.011,12.525-12.525 V99.466H74.749v125.123c0,7.515,5.01,12.525,12.525,12.525H99.8v43.808c0,10.02,8.736,18.758,18.758,18.758 c10.019,0,18.756-8.738,18.756-18.758v-43.808h25.051v43.808C162.364,290.941,171.102,299.679,181.122,299.679z"/>
    <path d="M256.214,224.589c10.02,0,18.756-8.737,18.756-18.758v-87.615c0-9.967-8.736-18.75-18.756-18.75 c-10.021,0-18.758,8.783-18.758,18.75v87.615C237.456,215.851,246.192,224.589,256.214,224.589z"/>
    <path d="M43.466,224.589c10.021,0,18.758-8.737,18.758-18.758v-87.615c0-9.967-8.736-18.75-18.758-18.75 c-10.02,0-18.756,8.783-18.756,18.75v87.615C24.71,215.851,33.446,224.589,43.466,224.589z"/>
    <path d="M209.899,1.89c-2.504-2.52-6.232-2.52-8.736,0l-16.799,16.743l-0.775,0.774 c-9.961-4.988-21.129-7.479-33.566-7.503c-0.061,0-0.121-0.002-0.182-0.002h-0.002c-0.063,0-0.121,0.002-0.184,0.002 c-12.436,0.024-23.604,2.515-33.564,7.503l-0.777-0.774L98.516,1.89c-2.506-2.52-6.232-2.52-8.736,0 c-2.506,2.506-2.506,6.225,0,8.729l16.25,16.253c-5.236,3.496-9.984,7.774-14.113,12.667C82.032,51.256,75.727,66.505,74.86,83.027 c-0.008,0.172-0.025,0.342-0.033,0.514c-0.053,1.125-0.078,2.256-0.078,3.391H224.93c0-1.135-0.027-2.266-0.078-3.391 c-0.008-0.172-0.025-0.342-0.035-0.514c-0.865-16.522-7.172-31.772-17.057-43.487c-4.127-4.893-8.877-9.171-14.113-12.667 l16.252-16.253C212.405,8.115,212.405,4.396,209.899,1.89z M118.534,65.063c-5.182,0-9.383-4.201-9.383-9.383 c0-5.182,4.201-9.383,9.383-9.383c5.182,0,9.383,4.201,9.383,9.383C127.917,60.862,123.716,65.063,118.534,65.063z M181.145,65.063 c-5.182,0-9.383-4.201-9.383-9.383c0-5.182,4.201-9.383,9.383-9.383c5.182,0,9.383,4.201,9.383,9.383 C190.528,60.862,186.327,65.063,181.145,65.063z"/>
  </svg>
);

const DownloadIcon = () => (
  <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeLinejoin="round">
    <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" /><polyline points="7 10 12 15 17 10" /><line x1="12" y1="15" x2="12" y2="3" />
  </svg>
);

// ─── Helpers ─────────────────────────────────────────────

type Asset = { id: number; name: string; browser_download_url: string; size: number };

const formatSize = (bytes: number) => {
  if (bytes > 1048576) return `${(bytes / 1048576).toFixed(1)} MB`;
  return `${(bytes / 1024).toFixed(0)} KB`;
};

const classifyAsset = (name: string): { os: string; arch: string; ext: string } | null => {
  const lower = name.toLowerCase();
  if (lower.endsWith('.apk')) {
    return { os: 'Android', arch: '', ext: 'Aplikasi (.apk)' };
  }
  if (lower.endsWith('.exe') || lower.endsWith('.msi')) {
    const arch = lower.includes('arm') ? 'ARM64' : 'x64';
    return { os: 'Windows', arch, ext: lower.endsWith('.exe') ? 'Installer (.exe)' : 'Installer (.msi)' };
  }
  if (lower.endsWith('.dmg')) {
    const arch = lower.includes('arm64') ? 'Apple Silicon' : 'Intel x64';
    return { os: 'macOS', arch, ext: `.dmg — ${arch}` };
  }
  if (lower.endsWith('.appimage')) {
    return { os: 'Linux', arch: 'x86_64', ext: 'AppImage' };
  }
  if (lower.endsWith('.deb')) {
    return { os: 'Linux', arch: 'x86_64', ext: 'Debian (.deb)' };
  }
  if (lower.endsWith('.rpm')) {
    return { os: 'Linux', arch: 'x86_64', ext: 'RPM (.rpm)' };
  }
  if (lower.endsWith('.tar.gz') || lower.endsWith('.zip')) {
    if (lower.includes('linux')) return { os: 'Linux', arch: 'x86_64', ext: 'Archive' };
    if (lower.includes('darwin') || lower.includes('mac')) return { os: 'macOS', arch: '', ext: 'Archive' };
    if (lower.includes('win')) return { os: 'Windows', arch: 'x64', ext: 'Portable (.zip)' };
  }
  return null;
};

const osIcon = (os: string) => {
  switch (os) {
    case 'Windows': return <WindowsIcon />;
    case 'macOS': return <MacIcon />;
    case 'Linux': return <LinuxIcon />;
    case 'Android': return <AndroidIcon />;
    default: return <DownloadIcon />;
  }
};

// ─── Component ───────────────────────────────────────────

const Download = () => {
  const [rakodaRelease, setRakodaRelease] = useState<any>(null);
  const [studioRelease, setStudioRelease] = useState<any>(null);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    fetch('https://api.github.com/repos/resitdc/rakoda-programming-language/releases')
      .then(res => { if (!res.ok) throw new Error('Not found'); return res.json(); })
      .then(releases => {
        if (Array.isArray(releases)) {
          const rakoda = releases.find((r: any) => r.tag_name.startsWith('v') && !r.tag_name.startsWith('studio-'));
          const studio = releases.find((r: any) => r.tag_name.startsWith('studio-'));
          if (rakoda) setRakodaRelease(rakoda);
          if (studio) setStudioRelease(studio);
        }
        setLoading(false);
      })
      .catch(() => setLoading(false));
  }, []);

  // Group CLI assets by OS
  const groupedRakoda: Record<string, { info: ReturnType<typeof classifyAsset>; asset: Asset }[]> = {};
  if (rakodaRelease?.assets) {
    for (const asset of rakodaRelease.assets) {
      const info = classifyAsset(asset.name);
      if (info) {
        if (!groupedRakoda[info.os]) groupedRakoda[info.os] = [];
        groupedRakoda[info.os].push({ info, asset });
      }
    }
  }

  const getStudioAssetUrl = (os: string) => {
    if (!studioRelease?.assets) return null;
    const asset = studioRelease.assets.find((a: any) => {
       const info = classifyAsset(a.name);
       return info && info.os === os;
    });
    return asset?.browser_download_url;
  };

  const osOrder = ['Windows', 'macOS', 'Linux'];

  return (
    <Layout title="Unduh" description="Unduh Rakoda Compiler dan RPL Studio">
      <main className={styles.page}>

        {/* Header */}
        <div className={styles.header}>
          <span className={styles.badge}>Open Source</span>
          <Heading as="h1" className={styles.title}>Unduh Rakoda</Heading>
          <p className={styles.subtitle}>
            Dapatkan compiler resmi dan IDE untuk mulai menulis kode dengan Bahasa Indonesia.
          </p>
        </div>

        {/* RPL Studio — full width highlight */}
        <section className={styles.studioSection}>
          <div className={styles.studioContent}>
            <div className={styles.studioLabel}>IDE Resmi</div>
            <Heading as="h2" className={styles.studioTitle}>
              RPL Studio
              {studioRelease && <span className={styles.versionBadge} style={{ marginLeft: '12px' }}>{studioRelease.tag_name}</span>}
            </Heading>
            <p className={styles.studioDesc}>
              Editor kode profesional dengan Autocompletion, HTTP Workspace,
              integrasi Database, dan eksekusi kode langsung di dalam aplikasi.
            </p>
          </div>
          <div className={styles.studioGrid}>
            <a href={getStudioAssetUrl('Windows') || "https://github.com/resitdc/rakoda-programming-language/releases"} target="_self" rel="noopener noreferrer" className={styles.platformCard}>
              <div className={styles.platformIcon}><WindowsIcon /></div>
              <div><div className={styles.platformName}>Windows</div><div className={styles.platformMeta}>Installer / Portable</div></div>
            </a>
            <a href={getStudioAssetUrl('macOS') || "https://github.com/resitdc/rakoda-programming-language/releases"} target="_self" rel="noopener noreferrer" className={styles.platformCard}>
              <div className={styles.platformIcon}><MacIcon /></div>
              <div><div className={styles.platformName}>macOS</div><div className={styles.platformMeta}>Installer (.dmg)</div></div>
            </a>
            <a href={getStudioAssetUrl('Linux') || "https://github.com/resitdc/rakoda-programming-language/releases"} target="_self" rel="noopener noreferrer" className={styles.platformCard}>
              <div className={styles.platformIcon}><LinuxIcon /></div>
              <div><div className={styles.platformName}>Linux</div><div className={styles.platformMeta}>AppImage / .deb</div></div>
            </a>
            <a href={getStudioAssetUrl('Android') || "https://github.com/resitdc/rakoda-programming-language/releases"} target="_self" rel="noopener noreferrer" className={styles.platformCard}>
              <div className={styles.platformIcon}><AndroidIcon /></div>
              <div><div className={styles.platformName}>Android</div><div className={styles.platformMeta}>Aplikasi (.apk)</div></div>
            </a>
          </div>
        </section>

        {/* CLI Compiler */}
        <section className={styles.cliSection}>
          <div className={styles.cliHeader}>
            <div>
              <Heading as="h2" className={styles.cliTitle}>Rakoda Compiler (CLI)</Heading>
              <p className={styles.cliDesc}>
                Jalankan kode Rakoda langsung dari terminal. Cocok untuk server, automation, atau pengembangan tanpa GUI.
              </p>
            </div>
            {rakodaRelease && (
              <span className={styles.versionBadge}>{rakodaRelease.tag_name}</span>
            )}
          </div>

          {loading ? (
            <div className={styles.loadingRow}>
              <div className={styles.loader} />
              <span>Mengambil data rilis terbaru...</span>
            </div>
          ) : rakodaRelease && Object.keys(groupedRakoda).length > 0 ? (
            <div className={styles.osGroups}>
              {osOrder.map(os => {
                const items = groupedRakoda[os];
                if (!items) return null;
                return (
                  <div key={os} className={styles.osGroup}>
                    <div className={styles.osGroupHeader}>
                      {osIcon(os)}
                      <span className={styles.osGroupName}>{os}</span>
                    </div>
                    <div className={styles.assetList}>
                      {items.map(({ info, asset }) => (
                        <a key={asset.id} href={asset.browser_download_url} className={styles.assetRow}>
                          <div className={styles.assetInfo}>
                            <span className={styles.assetExt}>{info.ext}</span>
                          </div>
                          <div className={styles.assetMeta}>
                            <span className={styles.assetSize}>{formatSize(asset.size)}</span>
                            <DownloadIcon />
                          </div>
                        </a>
                      ))}
                    </div>
                  </div>
                );
              })}
            </div>
          ) : (
            <a href="https://github.com/resitdc/rakoda-programming-language/releases" target="_blank" className="button button--secondary button--block">
              Lihat Rilis di GitHub
            </a>
          )}
        </section>

      </main>
    </Layout>
  );
};

export default Download;

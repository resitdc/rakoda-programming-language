import React, { useState, useEffect, type ReactNode } from 'react';
import clsx from 'clsx';
import Link from '@docusaurus/Link';
import useDocusaurusContext from '@docusaurus/useDocusaurusContext';
import useBaseUrl from '@docusaurus/useBaseUrl';
import Layout from '@theme/Layout';
import HomepageFeatures from '@site/src/components/HomepageFeatures';
import Heading from '@theme/Heading';
import styles from './index.module.css';

// Code examples for typing animation
const codes = [
  `buat nama = "Restu"\nbuat umur = 26\n\njika umur minimal 17 maka\ntampilkan \`\${nama} sudah punya KTP\`\njika tidak\ntampilkan \`\${nama} belum punya KTP\`\nselesai`,
  `buat buah = ["Apel", "Jeruk", "Pisang"]\n\nsetiap item di buah maka\ntampilkan "Saya suka " + item\nselesai`,
];

const KEYWORDS = ['buat', 'jika', 'maka', 'tidak', 'selesai', 'selama', 'setiap', 'di', 'minimal'];

const highlightRPL = (text: string) => {
  if (!text) return null;
  const regex = new RegExp(`(${KEYWORDS.join('|')}|\\s+|"[^"]*"|\`[^\`]*\`|[0-9]+)`);
  const tokens = text.split(regex);
  return tokens.map((token, i) => {
    if (!token) return null;
    if (KEYWORDS.includes(token)) return <span key={i} style={{ color: '#60a5fa' }}>{token}</span>;
    if (token === 'tampilkan') return <span key={i} style={{ color: '#34d399' }}>{token}</span>;
    if (/^[0-9]+$/.test(token)) return <span key={i} style={{ color: '#a78bfa' }}>{token}</span>;
    if (/^["`]/.test(token)) return <span key={i} style={{ color: '#fbbf24' }}>{token}</span>;
    return <span key={i} style={{ color: '#e2e8f0' }}>{token}</span>;
  });
};

const CustomCodeBlock = () => {
  const [copied, setCopied] = useState(false);
  const [codeIndex, setCodeIndex] = useState(0);
  const [charIndex, setCharIndex] = useState(0);

  const currentCode = codes[codeIndex];
  const isWaiting = charIndex >= currentCode.length;

  useEffect(() => {
    if (charIndex < currentCode.length) {
      const timeout = setTimeout(() => setCharIndex(c => c + 1), 40);
      return () => clearTimeout(timeout);
    } else {
      const timeout = setTimeout(() => {
        setCharIndex(0);
        setCodeIndex((c) => (c + 1) % codes.length);
      }, 4000);
      return () => clearTimeout(timeout);
    }
  }, [charIndex, codeIndex, currentCode.length]);

  const currentText = currentCode.substring(0, charIndex);
  const lines = currentText.split('\n');

  const handleCopy = () => {
    navigator.clipboard.writeText(codes[codeIndex]);
    setCopied(true);
    setTimeout(() => setCopied(false), 2000);
  };

  return (
    <div style={{ position: 'relative' }}>
      <button
        onClick={handleCopy}
        className="clean-btn"
        title="Salin Kode"
        style={{
          position: 'absolute', top: 10, right: 10, zIndex: 10,
          background: 'rgba(255,255,255,0.06)',
          border: '1px solid rgba(255,255,255,0.1)',
          color: 'rgba(255,255,255,0.5)', padding: '3px 10px',
          borderRadius: 6, cursor: 'pointer', fontSize: '0.75rem',
          fontFamily: 'var(--ifm-font-family-monospace)',
          transition: 'all 0.15s ease',
        }}
        onMouseOver={(e) => { e.currentTarget.style.color = '#fff'; e.currentTarget.style.background = 'rgba(255,255,255,0.1)'; }}
        onMouseOut={(e) => { e.currentTarget.style.color = 'rgba(255,255,255,0.5)'; e.currentTarget.style.background = 'rgba(255,255,255,0.06)'; }}
      >
        {copied ? 'Tersalin' : 'Salin'}
      </button>
      <div style={{ padding: '16px 14px', minHeight: 220 }}>
        {lines.map((line, i) => (
          <div key={i} style={{
            display: 'flex',
            fontFamily: 'JetBrains Mono, var(--ifm-font-family-monospace)',
            fontSize: '0.85rem', lineHeight: '1.6', minHeight: '1.6em',
          }}>
            <span style={{
              width: '1.75rem', textAlign: 'right', color: '#475569',
              marginRight: '1rem', userSelect: 'none', fontSize: '0.8rem',
            }}>{i + 1}</span>
            <span>{highlightRPL(line)}</span>
            {i === lines.length - 1 && !isWaiting && (
              <span style={{
                borderRight: '2px solid #60a5fa', marginLeft: 1,
                animation: 'blink 1s step-end infinite',
              }} />
            )}
          </div>
        ))}
      </div>
      <style>{`@keyframes blink { 50% { opacity: 0; } }`}</style>
    </div>
  );
};

const HomepageHeader = () => {
  const { siteConfig } = useDocusaurusContext();
  return (
    <header className={clsx('hero hero--primary', styles.heroBanner)}>
      <div className="container">
        <div className="row" style={{ alignItems: 'center' }}>
          <div className="col col--6" style={{ textAlign: 'left' }}>
            <img
              src={useBaseUrl('/img/rakoda-white.svg')}
              alt="Rakoda Logo"
              className={styles.heroLogo}
            />
            <Heading as="h1" className="hero__title" style={{ fontSize: '2.5rem', letterSpacing: '-0.03em', fontWeight: 800 }}>
              {siteConfig.title}
            </Heading>
            <p className={styles.heroSubtitle}>
              Bahasa pemrograman yang dirancang khusus dengan sintaks <strong>Bahasa Indonesia</strong>,
              membuat belajar logika pemrograman menjadi lebih mudah dan relevan.
            </p>
            <div className={styles.buttons} style={{ flexWrap: 'wrap' }}>
              <Link className="button button--secondary button--lg" to="/docs/intro">
                Mulai Belajar
              </Link>
              <Link
                className="button button--outline button--secondary button--lg"
                to="/download"
                style={{ color: 'white', borderColor: 'rgba(255,255,255,0.3)' }}>
                Unduh
              </Link>
            </div>
          </div>
          <div className="col col--6">
            <div className={styles.codeWindow}>
              <div className={styles.codeHeader}>
                <span className={styles.macDot} style={{ backgroundColor: '#ff5f56' }} />
                <span className={styles.macDot} style={{ backgroundColor: '#ffbd2e' }} />
                <span className={styles.macDot} style={{ backgroundColor: '#27c93f' }} />
                <span className={styles.codeTitle}>belajar_ngoding.rpl</span>
              </div>
              <CustomCodeBlock />
            </div>
          </div>
        </div>
      </div>
    </header>
  );
};

const Home = (): ReactNode => {
  const { siteConfig } = useDocusaurusContext();
  return (
    <Layout
      title={`Beranda | ${siteConfig.title}`}
      description="Rakoda adalah bahasa pemrograman berbahasa Indonesia yang dirancang untuk memudahkan pemula dalam belajar coding.">
      <HomepageHeader />
      <main>
        <HomepageFeatures />
      </main>
    </Layout>
  );
};

export default Home;

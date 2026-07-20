import React, { useState, useEffect, type ReactNode } from 'react';
import clsx from 'clsx';
import Link from '@docusaurus/Link';
import useDocusaurusContext from '@docusaurus/useDocusaurusContext';
import Layout from '@theme/Layout';
import HomepageFeatures from '@site/src/components/HomepageFeatures';
import Heading from '@theme/Heading';
import styles from './index.module.css';

let firstCode = `buat nama = "Restu"\n`;
firstCode += "buat umur = 26\n\n";
firstCode += "jika umur minimal 17 maka\n";
firstCode += 'tampilkan `${nama} sudah punya KTP`\n';
firstCode += "jika tidak\n";
firstCode += 'tampilkan `${nama} belum punya KTP`\n';
firstCode += "selesai";

let secondCode = `buat nama = "Restu"\n\n`;
secondCode += 'jika nama isinya "Restu"\n';
secondCode += 'tampilkan "Kamu Ganteng"\n';
secondCode += "jika tidak\n";
secondCode += 'tampilkan "Kamu Jelek"\n';
secondCode += "selesai";

const codes = [
  firstCode,
  secondCode,
];

const highlightRPL = (text: string) => {
  const kw = { color: '#569CD6' };
  const fn = { color: '#4EC9B0' };
  const st = { color: '#CE9178' };
  const nm = { color: '#B5CEA8' };
  const id = { color: '#D4D4D4' };

  if (!text) return null;

  const regex = /(buat|jika|maka|tidak|selesai|tampilkan|\s+|"[^"]*"|`[^`]*`|[0-9]+)/;
  const tokens = text.split(regex);
  return tokens.map((token, i) => {
    if (!token) return null;
    if (["buat", "jika", "maka", "tidak", "selesai"].includes(token)) return <span key={i} style={kw}>{token}</span>;
    if (token === "tampilkan") return <span key={i} style={fn}>{token}</span>;
    if (/^[0-9]+$/.test(token)) return <span key={i} style={nm}>{token}</span>;
    if (/^["`]/.test(token)) return <span key={i} style={st}>{token}</span>;
    return <span key={i} style={id}>{token}</span>;
  });
}

const CustomCodeBlock = () => {
  const [copied, setCopied] = useState(false);
  const [codeIndex, setCodeIndex] = useState(0);
  const [charIndex, setCharIndex] = useState(0);

  const currentCode = codes[codeIndex];
  const isWaiting = charIndex >= currentCode.length;

  useEffect(() => {
    if (charIndex < currentCode.length) {
      const timeout = setTimeout(() => {
        setCharIndex(c => c + 1);
      }, 40);
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

  const lineStyle = { display: 'flex', fontFamily: 'var(--ifm-font-family-monospace)', fontSize: '0.9rem', lineHeight: '1.5', minHeight: '1.5em' };
  const numStyle = { width: '2rem', textAlign: 'right' as const, color: '#858585', marginRight: '1rem', userSelect: 'none' as const };
  const id = { color: '#D4D4D4' }; 

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
          position: 'absolute',
          top: '12px',
          right: '12px',
          background: 'rgba(255, 255, 255, 0.1)',
          border: '1px solid rgba(255, 255, 255, 0.2)',
          color: '#fff',
          padding: '4px 8px',
          borderRadius: '4px',
          cursor: 'pointer',
          fontSize: '0.75rem',
          opacity: 0.8,
          transition: 'all 0.2s',
          zIndex: 10
        }}
        onMouseOver={(e) => { e.currentTarget.style.opacity = '1'; e.currentTarget.style.background = 'rgba(255, 255, 255, 0.2)'; }}
        onMouseOut={(e) => { e.currentTarget.style.opacity = '0.8'; e.currentTarget.style.background = 'rgba(255, 255, 255, 0.1)'; }}
      >
        {copied ? 'Tersalin!' : 'Salin'}
      </button>
      <div style={{ padding: '20px 16px', background: 'transparent', overflowX: 'auto', minHeight: '260px' }}>
        {lines.map((line, i) => (
          <div key={i} style={lineStyle}>
            <span style={numStyle}>{i + 1}</span>
            <span style={id}>{highlightRPL(line)}</span>
            {i === lines.length - 1 && !isWaiting && (
              <span style={{ borderRight: '2px solid #D4D4D4', marginLeft: '2px' }}></span>
            )}
          </div>
        ))}
      </div>
    </div>
  );
}

const HomepageHeader = () => {
  const {siteConfig} = useDocusaurusContext();
  return (
    <header className={clsx('hero hero--primary', styles.heroBanner)}>
      <div className="container">
        <div className="row" style={{ alignItems: 'center' }}>
          <div className="col col--6" style={{ textAlign: 'left' }}>
            <img 
              src="/img/rakoda-white.svg" 
              alt="Rakoda Logo" 
              className={styles.heroLogo}
            />
            <Heading as="h1" className="hero__title">
              {siteConfig.title}
            </Heading>
            <p className={styles.heroSubtitle}>
              Bahasa pemrograman yang dirancang khusus dengan sintaks <b>Bahasa Indonesia</b>, 
              membuat belajar logika pemrograman menjadi lebih mudah dan relevan!
            </p>
            <div className={styles.buttons} style={{ flexWrap: 'wrap' }}>
              <Link
                className="button button--secondary button--lg"
                to="/docs/intro">
                Mulai Belajar Sekarang
              </Link>
              <Link
                className="button button--outline button--secondary button--lg"
                to="/download"
                style={{ color: 'white', borderColor: 'white' }}>
                Unduh
              </Link>
              <Link
                className="button button--outline button--secondary button--lg"
                to="https://github.com/resitdc/rakoda-programming-language"
                style={{ color: 'white', borderColor: 'white' }}>
                GitHub
              </Link>
            </div>
          </div>
          <div className="col col--6">
            <div className={styles.codeWindow}>
              <div className={styles.codeHeader}>
                <span className={styles.macDot} style={{backgroundColor: '#ff5f56'}}></span>
                <span className={styles.macDot} style={{backgroundColor: '#ffbd2e'}}></span>
                <span className={styles.macDot} style={{backgroundColor: '#27c93f'}}></span>
                <span className={styles.codeTitle}>belajar_ngoding.rpl</span>
              </div>
              <CustomCodeBlock />
            </div>
          </div>
        </div>
      </div>
    </header>
  );
}

const Home = (): ReactNode => {
  const {siteConfig} = useDocusaurusContext();
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

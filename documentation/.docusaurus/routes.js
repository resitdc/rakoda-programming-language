import React from 'react';
import ComponentCreator from '@docusaurus/ComponentCreator';

export default [
  {
    path: '/__docusaurus/debug',
    component: ComponentCreator('/__docusaurus/debug', '5ff'),
    exact: true
  },
  {
    path: '/__docusaurus/debug/config',
    component: ComponentCreator('/__docusaurus/debug/config', '5ba'),
    exact: true
  },
  {
    path: '/__docusaurus/debug/content',
    component: ComponentCreator('/__docusaurus/debug/content', 'a2b'),
    exact: true
  },
  {
    path: '/__docusaurus/debug/globalData',
    component: ComponentCreator('/__docusaurus/debug/globalData', 'c3c'),
    exact: true
  },
  {
    path: '/__docusaurus/debug/metadata',
    component: ComponentCreator('/__docusaurus/debug/metadata', '156'),
    exact: true
  },
  {
    path: '/__docusaurus/debug/registry',
    component: ComponentCreator('/__docusaurus/debug/registry', '88c'),
    exact: true
  },
  {
    path: '/__docusaurus/debug/routes',
    component: ComponentCreator('/__docusaurus/debug/routes', '000'),
    exact: true
  },
  {
    path: '/download',
    component: ComponentCreator('/download', 'b81'),
    exact: true
  },
  {
    path: '/markdown-page',
    component: ComponentCreator('/markdown-page', '53a'),
    exact: true
  },
  {
    path: '/search',
    component: ComponentCreator('/search', '822'),
    exact: true
  },
  {
    path: '/docs',
    component: ComponentCreator('/docs', '877'),
    routes: [
      {
        path: '/docs',
        component: ComponentCreator('/docs', '09a'),
        routes: [
          {
            path: '/docs',
            component: ComponentCreator('/docs', 'e25'),
            routes: [
              {
                path: '/docs/category/dasar-pemrograman',
                component: ComponentCreator('/docs/category/dasar-pemrograman', '56a'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/docs/category/ekosistem--tooling',
                component: ComponentCreator('/docs/category/ekosistem--tooling', '636'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/docs/category/fitur-lanjut',
                component: ComponentCreator('/docs/category/fitur-lanjut', '1cc'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/docs/category/pengembangan-web',
                component: ComponentCreator('/docs/category/pengembangan-web', 'cd8'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/docs/category/pustaka-standar-stdlib',
                component: ComponentCreator('/docs/category/pustaka-standar-stdlib', '8e0'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/docs/dasar/fungsi',
                component: ComponentCreator('/docs/dasar/fungsi', '9a7'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/docs/dasar/operator',
                component: ComponentCreator('/docs/dasar/operator', '221'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/docs/dasar/percabangan',
                component: ComponentCreator('/docs/dasar/percabangan', '1b4'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/docs/dasar/perulangan',
                component: ComponentCreator('/docs/dasar/perulangan', '196'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/docs/dasar/tipe-data',
                component: ComponentCreator('/docs/dasar/tipe-data', 'ae4'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/docs/dasar/variabel',
                component: ComponentCreator('/docs/dasar/variabel', '8f5'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/docs/ekosistem/manajemen-paket',
                component: ComponentCreator('/docs/ekosistem/manajemen-paket', 'ba2'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/docs/ekosistem/perintah-cli',
                component: ComponentCreator('/docs/ekosistem/perintah-cli', 'ac7'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/docs/fitur-lanjut/kecerdasan-buatan',
                component: ComponentCreator('/docs/fitur-lanjut/kecerdasan-buatan', '5b5'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/docs/fitur-lanjut/modul-dan-import',
                component: ComponentCreator('/docs/fitur-lanjut/modul-dan-import', 'dab'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/docs/fitur-lanjut/penanganan-error',
                component: ComponentCreator('/docs/fitur-lanjut/penanganan-error', 'fba'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/docs/fitur-lanjut/tugas-dan-log',
                component: ComponentCreator('/docs/fitur-lanjut/tugas-dan-log', '522'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/docs/intro',
                component: ComponentCreator('/docs/intro', '89a'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/docs/pengembangan-web/database',
                component: ComponentCreator('/docs/pengembangan-web/database', '47f'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/docs/pengembangan-web/template-html',
                component: ComponentCreator('/docs/pengembangan-web/template-html', 'dc2'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/docs/pengembangan-web/web-dan-jaringan',
                component: ComponentCreator('/docs/pengembangan-web/web-dan-jaringan', '3d1'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/docs/pustaka-standar/inti-dan-tipe',
                component: ComponentCreator('/docs/pustaka-standar/inti-dan-tipe', 'dc2'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/docs/pustaka-standar/matematika-dan-teks',
                component: ComponentCreator('/docs/pustaka-standar/matematika-dan-teks', 'efc'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/docs/pustaka-standar/pustaka-standar',
                component: ComponentCreator('/docs/pustaka-standar/pustaka-standar', '6c7'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/docs/pustaka-standar/sistem-dan-berkas',
                component: ComponentCreator('/docs/pustaka-standar/sistem-dan-berkas', 'be5'),
                exact: true,
                sidebar: "tutorialSidebar"
              },
              {
                path: '/docs/pustaka-standar/waktu-dan-kripto',
                component: ComponentCreator('/docs/pustaka-standar/waktu-dan-kripto', 'ae2'),
                exact: true,
                sidebar: "tutorialSidebar"
              }
            ]
          }
        ]
      }
    ]
  },
  {
    path: '/',
    component: ComponentCreator('/', 'e5f'),
    exact: true
  },
  {
    path: '*',
    component: ComponentCreator('*'),
  },
];

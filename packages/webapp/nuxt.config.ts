// eslint-disable-next-line unicorn/import-style
import { dirname, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'
import { defineNuxtConfig } from 'nuxt/config'
import AutoImport from 'unplugin-auto-import/vite'
import { FileSystemIconLoader } from 'unplugin-icons/loaders'
import IconsResolver from 'unplugin-icons/resolver'
import Icons from 'unplugin-icons/vite'
import ViteComponents from 'unplugin-vue-components/vite'
import { loadEnv } from 'vite'
import { nodePolyfills } from 'vite-plugin-node-polyfills'

const env = loadEnv(process.env.NUXT_ENV || 'development', process.cwd(), '')

const __filename = fileURLToPath(import.meta.url)
const __dirname = dirname(__filename)

export default defineNuxtConfig({
  // @ts-expect-error...
  srcDir: 'src/',
  dir: {
    public: resolve(__dirname, 'public'),
  },
  ssr: false,
  devServer: {
    port: 3000,
  },
  css: [
    'bootstrap/dist/css/bootstrap.min.css',
    'assets/styles/app.scss',
    'assets/styles/bootstrap-custom.scss',
  ],
  components: [
    {
      path: '~/components',
      pathPrefix: false,
    },
  ],
  imports: {
    autoImport: true,
  },
  vue: {
    compilerOptions: {
      comments: false,
    },
  },
  vite: {
    plugins: [
      nodePolyfills({
        include: ['buffer'],
        globals: {
          Buffer: true,
        },
      }),

      AutoImport({
        imports: [
          'vue',
          '@vueuse/core',
          {
            '@gtm-support/vue-gtm': ['useGtm'],
            '@vueuse/core': ['isClient'],
          },
        ],
        dts: 'types/auto-imports.d.ts',
        dirs: [
          'hooks/**',
          'store/**',
          'features/toast',
        ],
        vueTemplate: true,
        injectAtEnd: true,
      }),
      // https://github.com/antfu/unplugin-icons
      Icons({
        compiler: 'vue3',
        autoInstall: true,
        // transform(svg) {
        //   return svg.replace(/^<svg /, '<svg fill="currentColor" ')
        // },
        customCollections: {
          app: FileSystemIconLoader('./src/assets/img/icons'),
          metrics: FileSystemIconLoader('./src/assets/img/metrics'),
        },
        // iconCustomizer(collection, icon, props) {
        //   if (collection === 'app') {
        //     props.class = 'app-icon'
        //   }
        // },
      }),
      ViteComponents({
        dirs: ['components'],
        resolvers: [
          IconsResolver({
            customCollections: ['app', 'metrics'],
          }),
        ],
        dts: 'types/components.d.ts',
      }),
    ],
    esbuild: {
      legalComments: 'none',
    },
    css: {
      preprocessorOptions: {
        scss: {
          additionalData: `@import "~/assets/styles/variables.scss";`, // global variables
          silenceDeprecations: ['import'],
          quietDeps: true,
          logger: {
            warn: () => {},
          },
          // api: 'modern-compiler',
          // Optimize for production
          // outputStyle: isProd ? 'compressed' : 'expanded',
          // sourceMap: isDev,
        },
      },
    },
    resolve: {
      alias: {
        'buffer': 'buffer',
        'node:buffer': 'buffer',
        '@stellar-client': resolve(__dirname, 'src/client'),
      },
      dedupe: [
        'react',
        'react-dom',
        'bn.js',
        'bs58',
        'lodash',
        'buffer-layout',
      ],
    },
    optimizeDeps: {
      include: [
        'bn.js',
        'buffer',
        'js-cookie',
        'mitt',
        'axios',
        'chart.js',
        'chartjs-plugin-annotation',
        'chartjs-adapter-luxon',
        '@creit.tech/stellar-wallets-kit',
        'fastestsmallesttextencoderdecoder',
      ],
      // exclude: ['bootstrap-vue-next'],
    },
    build: {
      cssCodeSplit: true,
      terserOptions: {
        format: {
          comments: false,
        },
      },
    },
  },

  experimental: {
    // inlineSSRStyles: false,
  },

  plugins: [
    // '~/plugins/gsap.ts',
    // '~/plugins/lotttie.ts',
  ],

  modules: [
    '@bootstrap-vue-next/nuxt',
    '@vueuse/nuxt',
    '@pinia/nuxt',
    '@nuxtjs/i18n',
    '@vite-pwa/nuxt',
  ],

  i18n: {
    strategy: 'no_prefix',
    defaultLocale: 'en',
    lazy: true,
    langDir: '../src/locales',
    locales: [
      { code: 'en', name: 'English', file: 'en.json' },
      { code: 'ua', name: 'Ukraine', file: 'ua.json' },
    ],
    detectBrowserLanguage: {
      useCookie: true,
      cookieKey: 'i18n_locale',
      alwaysRedirect: false,
      fallbackLocale: 'en',
    },
  },

  runtimeConfig: {
    public: {
      PASSWORD_PROTECT: env.NUXT_PUBLIC_PASSWORD_PROTECT,
      COOKIE_DOMAIN: env.NUXT_PUBLIC_COOKIE_DOMAIN ?? 'localhost',
      NUXT_PUBLIC_RPC: env.NUXT_PUBLIC_RPC,
      ALULA_CLIENT_NETWORK: env.NUXT_PUBLIC_NETWORK,
    },
  },
  routeRules: {
    '/': { isr: 60 },
    '/assets/**': { headers: { 'cache-control': 'public,max-age=31536000,immutable' } },
  },
  // debug: true,
  nitro: {
    logLevel: 'debug',
    prerender: {
      autoSubfolderIndex: false,
    },
    compressPublicAssets: true,
    // preset: 'node',
    // devErrorHandler: true,
    // prerender: {
    //   failOnError: false,
    // },
  },
  pwa: {
    registerType: 'autoUpdate',
    workbox: {
      skipWaiting: true,
      clientsClaim: true,
      globPatterns: ['**/*.{js,css,webp,png,svg,gif,ico,html,json,txt}'],
      maximumFileSizeToCacheInBytes: 5_000_000,
      navigateFallback: null,
      // runtimeCaching: [
      //   {
      //     urlPattern: /^https:\/\/www\.googletagmanager\.com\/gtm\.js/,
      //     handler: 'CacheFirst',
      //     options: {
      //       cacheName: 'gtm',
      //       expiration: {
      //         maxEntries: 30,
      //         maxAgeSeconds: 60 * 60 * 24 * 365, // 1 год
      //       },
      //       cacheableResponse: {
      //         statuses: [0, 200],
      //       },
      //     },
      //   },
      // ],
    },
    manifest: {
      name: env.NUXT_PUBLIC_APP_NAME ?? 'Alula',
      short_name:
        env.NUXT_PUBLIC_APP_SHORT_NAME
        ?? env.NUXT_PUBLIC_APP_NAME
        ?? 'Alula',
      description: env.NUXT_PUBLIC_APP_DESCRIPTION,
      theme_color: '#ffffff',
      display: 'standalone',
      icons: [
        {
          src: '/pwa-192x192.png',
          sizes: '192x192',
          type: 'image/png',
        },
        {
          src: '/pwa-512x512.png',
          sizes: '512x512',
          type: 'image/png',
        },
        {
          src: '/pwa-512x512.png',
          sizes: '512x512',
          type: 'image/png',
          purpose: 'any maskable',
        },
      ],
    },
    includeAssets: [
      '/img/apple-touch-icon.png',
      '/favicon.svg',
      '/favicon.ico',
      '/robots.txt',
    ],
  },

  app: {
    trailingSlash: 'remove',

    // remove prefetch from all nuxt-link
    pageTransition: false,

    baseURL: env.NUXT_PUBLIC_BASE_URL || '/',

    head: {
      htmlAttrs: {
        lang: 'en',
      },
      title: env.NUXT_PUBLIC_APP_TITLE ?? 'Alula',
      link: [
        { rel: 'icon', type: 'image/svg+xml', href: '/favicon-light.svg', media: '(prefers-color-scheme: light)' },
        { rel: 'icon', type: 'image/svg+xml', href: '/favicon.svg', media: '(prefers-color-scheme: dark)' },
        // Preconnect wallet connect
        { rel: 'preconnect', href: 'https://walletconnect.com', crossorigin: '' },
        { rel: 'preconnect', href: 'https://verify.walletconnect.com', crossorigin: '' },
      ],
      meta: [
        { name: 'description', content: env.NUXT_PUBLIC_APP_DESCRIPTION },
        // Open Graph tags (Facebook, Telegram, LinkedIn, etc.)
        { property: 'og:title', content: env.NUXT_PUBLIC_APP_TITLE },
        { property: 'og:description', content: env.NUXT_PUBLIC_APP_DESCRIPTION },
        { property: 'og:type', content: 'website' },
        { property: 'og:url', content: env.NUXT_PUBLIC_APP_URL },
        {
          property: 'og:image',
          content: `${env.NUXT_PUBLIC_APP_URL}/og-image-1200x630.png`,
        },
      ],
      script: [
        {
          innerHTML: `
             (function() {
              const prefersDark = window.matchMedia && window.matchMedia('(prefers-color-scheme: dark)').matches
              const setting = localStorage.getItem('vueuse-color-scheme') || 'auto'
              const isDarkMode = setting === 'dark' || (prefersDark && setting !== 'light')
              document?.body?.classList.toggle('body--dark', isDarkMode)
              document?.body?.classList.toggle('body--light', !isDarkMode)
            })();
          `,
          type: 'text/javascript',
          body: true,
        } as any,
        {
          innerHTML: `
              (function () {
                function isWindows() {
                  if (navigator.userAgentData && navigator.userAgentData.platform) {
                    return navigator.userAgentData.platform === 'Windows'
                  }
                  return navigator.userAgent.includes('Windows')
                }
                if (isWindows()) {
                  document.documentElement.classList.add('is-windows')
                }
              })()
          `,
          type: 'text/javascript',
          body: true,
        },
        {
          src: 'https://t.contentsquare.net/uxa/2baae6781cf55.js',
          async: true,
        },
      ],
    },
  },

  compatibilityDate: '2025-03-12',
})

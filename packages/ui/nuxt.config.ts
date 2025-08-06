// eslint-disable-next-line unicorn/import-style
import { dirname, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'
// import { nodePolyfills } from 'vite-plugin-node-polyfills'
import { defineNuxtConfig } from 'nuxt/config'
import AutoImport from 'unplugin-auto-import/vite'
import { FileSystemIconLoader } from 'unplugin-icons/loaders'
import IconsResolver from 'unplugin-icons/resolver'
import Icons from 'unplugin-icons/vite'
import ViteComponents from 'unplugin-vue-components/vite'
import { loadEnv } from 'vite'

const env = loadEnv(process.env.NUXT_ENV || 'development', process.cwd(), '')

const __filename = fileURLToPath(import.meta.url)
const __dirname = dirname(__filename)

export default defineNuxtConfig({
  srcDir: 'src/',
  dir: {
    public: resolve(__dirname, 'public'),
  },
  ssr: env.NODE_ENV !== 'development',
  devServer: {
    port: 3000,
  },
  css: [
    'bootstrap/dist/css/bootstrap.min.css',
    'assets/styles/app.scss',
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
  vite: {
    plugins: [
      // https://github.com/davidmyersdev/vite-plugin-node-polyfills
      // nodePolyfills(),

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
        viteOptimizeDeps: true,
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
            customCollections: ['app'],
          }),
        ],
        dts: 'types/components.d.ts',
      }),
    ],
    esbuild: {
      legalComments: 'none',
    },
    build: {
      terserOptions: {
        format: {
          comments: false,
        },
      },
    },
    css: {
      preprocessorOptions: {
        scss: {
          additionalData: `@import "~/assets/styles/variables.scss";`, // global variables
          silenceDeprecations: ['mixed-decls', 'color-functions', 'global-builtin', 'import', 'legacy-js-api'],
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
    langDir: resolve(__dirname, 'src/locales'),
    locales: [
      { code: 'en', name: 'English', file: 'en.json' },
      { code: 'ua', name: 'Ukraine', file: 'ua.json' },
    ],
  },

  runtimeConfig: {
    public: {
      PASSWORD_PROTECT: env.NUXT_PUBLIC_PASSWORD_PROTECT,
      COOKIE_DOMAIN: env.NUXT_PUBLIC_COOKIE_DOMAIN ?? 'localhost',
      NUXT_PUBLIC_RPC: env.NUXT_PUBLIC_RPC,
      JLEND_CLIENT_NETWORK: env.NUXT_PUBLIC_NETWORK,
    },
  },
  // debug: true,
  nitro: {
    logLevel: 'debug',
    prerender: {
      autoSubfolderIndex: false,
    },
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
      name: env.NUXT_PUBLIC_APP_NAME ?? 'Jlend',
      short_name:
        env.NUXT_PUBLIC_APP_SHORT_NAME
        ?? env.NUXT_PUBLIC_APP_NAME
        ?? 'JLend',
      description: env.NUXT_PUBLIC_APP_DESCRIPTION,
      theme_color: '#ffffff',
      display: 'standalone',
      icons: [
        {
          src: '/img/pwa-192x192.png',
          sizes: '192x192',
          type: 'image/png',
        },
        {
          src: '/img/pwa-512x512.png',
          sizes: '512x512',
          type: 'image/png',
        },
        {
          src: '/img/pwa-512x512.png',
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
    // @ts-expect-error...
    trailingSlash: 'remove',

    baseURL: env.NUXT_PUBLIC_BASE_URL || '/',

    head: {
      title: env.NUXT_PUBLIC_APP_TITLE ?? 'JLend',
      link: [
        { rel: 'icon', type: 'image/svg+xml', href: '/favicon.svg' },
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
      ],
    },
  },

  compatibilityDate: '2025-03-12',
})

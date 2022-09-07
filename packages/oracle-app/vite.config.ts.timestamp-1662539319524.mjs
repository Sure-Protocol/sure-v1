var __defProp = Object.defineProperty;
var __getOwnPropDesc = Object.getOwnPropertyDescriptor;
var __getOwnPropNames = Object.getOwnPropertyNames;
var __hasOwnProp = Object.prototype.hasOwnProperty;
var __esm = (fn, res) => function __init() {
  return fn && (res = (0, fn[__getOwnPropNames(fn)[0]])(fn = 0)), res;
};
var __export = (target, all) => {
  for (var name2 in all)
    __defProp(target, name2, { get: all[name2], enumerable: true });
};
var __copyProps = (to, from, except, desc) => {
  if (from && typeof from === "object" || typeof from === "function") {
    for (let key of __getOwnPropNames(from))
      if (!__hasOwnProp.call(to, key) && key !== except)
        __defProp(to, key, { get: () => from[key], enumerable: !(desc = __getOwnPropDesc(from, key)) || desc.enumerable });
  }
  return to;
};
var __toCommonJS = (mod) => __copyProps(__defProp({}, "__esModule", { value: true }), mod);

// package.json
var package_exports = {};
__export(package_exports, {
  default: () => package_default,
  dependencies: () => dependencies,
  devDependencies: () => devDependencies,
  name: () => name,
  private: () => private2,
  scripts: () => scripts,
  type: () => type,
  version: () => version
});
var name, version, private2, type, scripts, devDependencies, dependencies, package_default;
var init_package = __esm({
  "package.json"() {
    name = "oracle-app";
    version = "0.0.28";
    private2 = true;
    type = "module";
    scripts = {
      dev: "SURE_ENV=dev vite dev",
      build: "SURE_ENV=dev  vite build",
      preview: "vite preview",
      test: "playwright test",
      check: "svelte-kit sync && svelte-check --tsconfig ./tsconfig.json",
      "check:watch": "svelte-kit sync && svelte-check --tsconfig ./tsconfig.json --watch",
      lint: "prettier --check . && eslint .",
      format: "prettier --write ."
    };
    devDependencies = {
      "@esbuild-plugins/node-globals-polyfill": "^0.1.1",
      "@playwright/test": "^1.25.0",
      "@rollup/plugin-inject": "^4.0.4",
      "@sveltejs/adapter-auto": "next",
      "@sveltejs/adapter-node": "^1.0.0-next.88",
      "@sveltejs/kit": "next",
      "@sveltejs/vite-plugin-svelte": "^1.0.5",
      "@tsconfig/svelte": "^3.0.0",
      "@types/cookie": "^0.5.1",
      "@typescript-eslint/eslint-plugin": "^5.27.0",
      "@typescript-eslint/parser": "^5.27.0",
      eslint: "^8.16.0",
      "eslint-config-prettier": "^8.3.0",
      "eslint-plugin-svelte3": "^4.0.0",
      "node-sass": "^7.0.1",
      prettier: "^2.6.2",
      "prettier-plugin-svelte": "^2.7.0",
      "rollup-plugin-node-polyfills": "^0.2.1",
      svelte: "^3.46.0",
      "svelte-check": "^2.7.1",
      "svelte-preprocess": "^4.10.7",
      "svelte-steps": "^2.3.5",
      tslib: "^2.4.0",
      typescript: "^4.7.4",
      vite: "^3.0.8"
    };
    dependencies = {
      "@emotion/css": "^11.10.0",
      "@fontsource/fira-mono": "^4.5.0",
      "@saberhq/solana-contrib": "^1.14.4",
      "@solana/spl-token": "^0.3.4",
      "@solana/wallet-adapter-base": "^0.9.16",
      "@solana/wallet-adapter-wallets": "^0.18.5",
      "@solana/web3.js": "^1.54.0",
      "@surec/oracle": "workspace:^",
      "@svelte-on-solana/wallet-adapter-anchor": "^1.0.16-alpha.0",
      "@svelte-on-solana/wallet-adapter-core": "^1.0.8-alpha.0",
      "@svelte-on-solana/wallet-adapter-ui": "^1.0.20-alpha.0",
      "bn.js": "^5.2.1",
      cookie: "^0.4.1",
      "decimal.js": "3.0.0",
      sass: "^1.54.5"
    };
    package_default = {
      name,
      version,
      private: private2,
      type,
      scripts,
      devDependencies,
      dependencies
    };
  }
});

// vite.config.ts
import { sveltekit } from "@sveltejs/kit/vite";
import { NodeGlobalsPolyfillPlugin } from "@esbuild-plugins/node-globals-polyfill";
import path from "path";
import inject from "@rollup/plugin-inject";
import nodePolyfills from "rollup-plugin-node-polyfills";
var pkg = (init_package(), __toCommonJS(package_exports));
var config = {
  plugins: [sveltekit()],
  logLevel: "warn",
  ssr: {
    noExternal: Object.keys(pkg.dependencies || {})
  },
  optimizeDeps: {
    include: ["@solana/web3.js", "buffer", "@saberhq/solana-contrib"],
    esbuildOptions: {
      target: "esnext",
      plugins: [NodeGlobalsPolyfillPlugin({ buffer: true })]
    }
  },
  resolve: {
    alias: {
      $assets: path.resolve("src/assets"),
      stream: "rollup-plugin-node-polyfills/polyfills/stream"
    }
  },
  define: {
    "process.env.BROWSER": true,
    "process.env.NODE_DEBUG": JSON.stringify(""),
    "process.env.SURE_ENV": JSON.stringify(process.env.SURE_ENV)
  },
  build: {
    target: "esnext",
    commonjsOptions: {
      transformMixedEsModules: false
    },
    rollupOptions: {
      plugins: [inject({ Buffer: ["buffer", "Buffer"] }), nodePolyfills({ crypto: true })]
    }
  }
};
var vite_config_default = config;
export {
  vite_config_default as default
};
//# sourceMappingURL=data:application/json;base64,ewogICJ2ZXJzaW9uIjogMywKICAic291cmNlcyI6IFsidml0ZS5jb25maWcudHMiXSwKICAic291cmNlc0NvbnRlbnQiOiBbImNvbnN0IF9fdml0ZV9pbmplY3RlZF9vcmlnaW5hbF9kaXJuYW1lID0gXCIvVXNlcnMva3Jpc3RvZmZlcmhvdmxhbmRiZXJnL2dpdGh1Yi9zdXJlLXByb3RvY29sL3N1cmUvcGFja2FnZXMvb3JhY2xlLWFwcFwiO2NvbnN0IF9fdml0ZV9pbmplY3RlZF9vcmlnaW5hbF9maWxlbmFtZSA9IFwiL1VzZXJzL2tyaXN0b2ZmZXJob3ZsYW5kYmVyZy9naXRodWIvc3VyZS1wcm90b2NvbC9zdXJlL3BhY2thZ2VzL29yYWNsZS1hcHAvdml0ZS5jb25maWcudHNcIjtjb25zdCBfX3ZpdGVfaW5qZWN0ZWRfb3JpZ2luYWxfaW1wb3J0X21ldGFfdXJsID0gXCJmaWxlOi8vL1VzZXJzL2tyaXN0b2ZmZXJob3ZsYW5kYmVyZy9naXRodWIvc3VyZS1wcm90b2NvbC9zdXJlL3BhY2thZ2VzL29yYWNsZS1hcHAvdml0ZS5jb25maWcudHNcIjtpbXBvcnQgeyBzdmVsdGVraXQgfSBmcm9tICdAc3ZlbHRlanMva2l0L3ZpdGUnO1xuaW1wb3J0IHR5cGUgeyBVc2VyQ29uZmlnIH0gZnJvbSAndml0ZSc7XG5pbXBvcnQgeyBOb2RlR2xvYmFsc1BvbHlmaWxsUGx1Z2luIH0gZnJvbSAnQGVzYnVpbGQtcGx1Z2lucy9ub2RlLWdsb2JhbHMtcG9seWZpbGwnO1xuaW1wb3J0IHBhdGggZnJvbSAncGF0aCc7XG5pbXBvcnQgaW5qZWN0IGZyb20gJ0Byb2xsdXAvcGx1Z2luLWluamVjdCc7XG5pbXBvcnQgbm9kZVBvbHlmaWxscyBmcm9tICdyb2xsdXAtcGx1Z2luLW5vZGUtcG9seWZpbGxzJztcbmNvbnN0IHBrZyA9IHJlcXVpcmUoJy4vcGFja2FnZS5qc29uJyk7XG5cbi8qKiBAdHlwZSB7aW1wb3J0KCd2aXRlJykuVXNlckNvbmZpZ30gKi9cbmNvbnN0IGNvbmZpZzogVXNlckNvbmZpZyA9IHtcblx0cGx1Z2luczogW3N2ZWx0ZWtpdCgpXSxcblx0bG9nTGV2ZWw6ICd3YXJuJyxcblx0c3NyOiB7XG5cdFx0bm9FeHRlcm5hbDogT2JqZWN0LmtleXMocGtnLmRlcGVuZGVuY2llcyB8fCB7fSlcblx0fSxcblx0b3B0aW1pemVEZXBzOiB7XG5cdFx0aW5jbHVkZTogWydAc29sYW5hL3dlYjMuanMnLCAnYnVmZmVyJywgJ0BzYWJlcmhxL3NvbGFuYS1jb250cmliJ10sXG5cdFx0ZXNidWlsZE9wdGlvbnM6IHtcblx0XHRcdHRhcmdldDogJ2VzbmV4dCcsXG5cdFx0XHRwbHVnaW5zOiBbTm9kZUdsb2JhbHNQb2x5ZmlsbFBsdWdpbih7IGJ1ZmZlcjogdHJ1ZSB9KV1cblx0XHR9XG5cdH0sXG5cdHJlc29sdmU6IHtcblx0XHRhbGlhczoge1xuXHRcdFx0JGFzc2V0czogcGF0aC5yZXNvbHZlKCdzcmMvYXNzZXRzJyksXG5cdFx0XHRzdHJlYW06ICdyb2xsdXAtcGx1Z2luLW5vZGUtcG9seWZpbGxzL3BvbHlmaWxscy9zdHJlYW0nXG5cdFx0fVxuXHR9LFxuXHRkZWZpbmU6IHtcblx0XHQncHJvY2Vzcy5lbnYuQlJPV1NFUic6IHRydWUsXG5cdFx0J3Byb2Nlc3MuZW52Lk5PREVfREVCVUcnOiBKU09OLnN0cmluZ2lmeSgnJyksXG5cdFx0J3Byb2Nlc3MuZW52LlNVUkVfRU5WJzogSlNPTi5zdHJpbmdpZnkocHJvY2Vzcy5lbnYuU1VSRV9FTlYpXG5cdH0sXG5cdGJ1aWxkOiB7XG5cdFx0dGFyZ2V0OiAnZXNuZXh0Jyxcblx0XHRjb21tb25qc09wdGlvbnM6IHtcblx0XHRcdHRyYW5zZm9ybU1peGVkRXNNb2R1bGVzOiBmYWxzZVxuXHRcdH0sXG5cdFx0cm9sbHVwT3B0aW9uczoge1xuXHRcdFx0cGx1Z2luczogW2luamVjdCh7IEJ1ZmZlcjogWydidWZmZXInLCAnQnVmZmVyJ10gfSksIG5vZGVQb2x5ZmlsbHMoeyBjcnlwdG86IHRydWUgfSldXG5cdFx0fVxuXHR9XG59O1xuXG5leHBvcnQgZGVmYXVsdCBjb25maWc7XG4iXSwKICAibWFwcGluZ3MiOiAiOzs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7Ozs7OztBQUFnWixTQUFTLGlCQUFpQjtBQUUxYSxTQUFTLGlDQUFpQztBQUMxQyxPQUFPLFVBQVU7QUFDakIsT0FBTyxZQUFZO0FBQ25CLE9BQU8sbUJBQW1CO0FBQzFCLElBQU0sTUFBTTtBQUdaLElBQU0sU0FBcUI7QUFBQSxFQUMxQixTQUFTLENBQUMsVUFBVSxDQUFDO0FBQUEsRUFDckIsVUFBVTtBQUFBLEVBQ1YsS0FBSztBQUFBLElBQ0osWUFBWSxPQUFPLEtBQUssSUFBSSxnQkFBZ0IsQ0FBQyxDQUFDO0FBQUEsRUFDL0M7QUFBQSxFQUNBLGNBQWM7QUFBQSxJQUNiLFNBQVMsQ0FBQyxtQkFBbUIsVUFBVSx5QkFBeUI7QUFBQSxJQUNoRSxnQkFBZ0I7QUFBQSxNQUNmLFFBQVE7QUFBQSxNQUNSLFNBQVMsQ0FBQywwQkFBMEIsRUFBRSxRQUFRLEtBQUssQ0FBQyxDQUFDO0FBQUEsSUFDdEQ7QUFBQSxFQUNEO0FBQUEsRUFDQSxTQUFTO0FBQUEsSUFDUixPQUFPO0FBQUEsTUFDTixTQUFTLEtBQUssUUFBUSxZQUFZO0FBQUEsTUFDbEMsUUFBUTtBQUFBLElBQ1Q7QUFBQSxFQUNEO0FBQUEsRUFDQSxRQUFRO0FBQUEsSUFDUCx1QkFBdUI7QUFBQSxJQUN2QiwwQkFBMEIsS0FBSyxVQUFVLEVBQUU7QUFBQSxJQUMzQyx3QkFBd0IsS0FBSyxVQUFVLFFBQVEsSUFBSSxRQUFRO0FBQUEsRUFDNUQ7QUFBQSxFQUNBLE9BQU87QUFBQSxJQUNOLFFBQVE7QUFBQSxJQUNSLGlCQUFpQjtBQUFBLE1BQ2hCLHlCQUF5QjtBQUFBLElBQzFCO0FBQUEsSUFDQSxlQUFlO0FBQUEsTUFDZCxTQUFTLENBQUMsT0FBTyxFQUFFLFFBQVEsQ0FBQyxVQUFVLFFBQVEsRUFBRSxDQUFDLEdBQUcsY0FBYyxFQUFFLFFBQVEsS0FBSyxDQUFDLENBQUM7QUFBQSxJQUNwRjtBQUFBLEVBQ0Q7QUFDRDtBQUVBLElBQU8sc0JBQVE7IiwKICAibmFtZXMiOiBbXQp9Cg==

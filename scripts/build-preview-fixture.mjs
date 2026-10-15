import {build} from 'vite';import {svelte} from '@sveltejs/vite-plugin-svelte';import path from 'node:path';
await build({configFile:false,plugins:[svelte()],build:{outDir:'docs/verification/preview-fixture-dist',rollupOptions:{input:path.resolve('docs/verification/preview-fixture/index.html')}},logLevel:'error'});

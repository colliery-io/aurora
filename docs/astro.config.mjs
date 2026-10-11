import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';
import mermaid from 'astro-mermaid';
import starlightAurora from 'starlight-aurora';

// The Pages workflow sets DOCS_SITE and DOCS_BASE. On a pull request and
// locally they are empty, and the site is at /.
const site = process.env.DOCS_SITE || undefined;
const base = process.env.DOCS_BASE || '/';

const PROJECT = 'Aurora';
const REPO = 'https://github.com/colliery-io/aurora';

export default defineConfig({
	site,
	base,
	// Mermaid is one large chunk, loaded only on a page with a diagram.
	vite: { build: { chunkSizeWarningLimit: 4096 } },
	integrations: [
		// Before Starlight. Light diagrams in the light theme; dark diagrams in
		// the dark theme and in hyper (the default theme applies to every theme
		// that astro-mermaid does not know).
		mermaid({ theme: 'dark', autoTheme: true }),
		starlight({
			title: PROJECT,
			description: 'Aurora, the Colliery design system for Leptos.',
			social: [{ icon: 'github', label: 'GitHub', href: REPO }],
			plugins: [starlightAurora({ hyper: true })],
			sidebar: [
				{ label: 'Tutorials', items: [{ autogenerate: { directory: 'tutorials' } }] },
				{ label: 'How-to guides', items: [{ autogenerate: { directory: 'how-to' } }] },
				{ label: 'Reference', items: [{ autogenerate: { directory: 'reference' } }] },
				{ label: 'Explanation', items: [{ autogenerate: { directory: 'explanation' } }] },
			],
		}),
	],
});

import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';

export default defineConfig({
  site: 'https://fullzer4.github.io/flashvm',
  base: '/flashvm',
  contentLayer: true,
  integrations: [
    starlight({
      title: 'flashVM',
      tagline: 'Run Python snippets inside microVMs (libkrun/krunvm).',
      description: 'Run short Python snippets in a KVM microVM via krunvm/libkrun. Ships an embedded OCI image for zero-setup.',
      social: [
        { icon: 'github', label: 'GitHub', href: 'https://github.com/fullzer4/flashvm' }
      ],
      editLink: {
        baseUrl: 'https://github.com/fullzer4/flashvm/edit/main/docs/src/content/docs/'
      },
      lastUpdated: true,
      sidebar: [
        {
          label: 'Getting Started',
          items: [
            { label: 'Overview', link: '/' },
            { label: 'Installation & Requirements', link: '/install' },
            { label: 'Quickstart', link: '/quickstart' }
          ]
        },
        {
          label: 'Usage',
          items: [
            { label: 'Python API', link: '/api' },
            { label: 'Artifacts', link: '/usage/artifacts' },
            { label: 'Embedded Image & Storage', link: '/usage/image' },
            { label: 'Troubleshooting', link: '/troubleshooting' }
          ]
        },
        {
          label: 'Guides',
          items: [
            { label: 'Examples', link: '/examples' },
            { label: 'Isolation & Security', link: '/guides/isolation' },
            { label: 'Internals', link: '/internals' }
          ]
        },
        {
          label: 'Reference',
          items: [
            { label: 'Result Schema', link: '/reference/result' },
            { label: 'Host Tooling (krunvm/buildah/skopeo)', link: '/reference/host-tools' },
            { label: 'Transport Syntax (oci:/containers-storage:)', link: '/reference/transports' }
          ]
        },
        {
          label: 'Project',
          items: [
            { label: 'Contributing', link: '/contributing' },
            { label: 'Changelog', link: '/changelog' },
            { label: 'License', link: '/license' }
          ]
        }
      ]
    })
  ]
});

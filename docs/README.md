# ScopeLint static site

This directory contains a small, static marketing/docs page for ScopeLint. It is designed to be served directly via GitHub Pages.

## Structure

- `index.html` – single page with hero, why, features, usage, examples, CI, and FAQ sections.
- `styles.css` – global styles for layout, typography, and cards/code blocks.
- `main.js` – light progressive enhancement (smooth scrolling between sections).

## Hosting with GitHub Pages

To serve this site at `<username>.github.io/scopelint` or a custom domain:

1. In the GitHub repository settings for `scopelint`, open **Pages**.
2. Set the **Source** to `Deploy from a branch` and choose the default branch and `/docs` folder.
3. Save. GitHub will build and host the contents of this directory as a static site.

If you later point a custom domain such as `scopelint.dev` at GitHub Pages, you can add a `CNAME` file in this directory with the domain name as the only line. Until then, no extra configuration is required.


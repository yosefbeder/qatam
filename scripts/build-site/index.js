import { unified } from 'unified';
import remarkParse from 'remark-parse';
import remarkToc from 'remark-toc';
import remarkRehype from 'remark-rehype';
import rehypeSlug from 'rehype-slug';
import rehypeDocument from 'rehype-document';
import rehypeStringify from 'rehype-stringify';
import { readSync, writeSync } from 'to-vfile';
import remarkStringify from 'remark-stringify';
import { exec } from 'child_process';

// Updating toc in README.md
unified()
  .use(remarkParse)
  .use(remarkToc, { heading: 'المحتويات', tight: true })
  .use(remarkStringify)
  .process(readSync('README.md'))
  .then(
    file => {
      writeSync(file);
    },
    err => {
      throw err;
    },
  );

// Building www/public/index.html
unified()
  .use(remarkParse)
  .use(remarkRehype)
  .use(rehypeSlug)
  .use(rehypeDocument, {
    title: 'قتام',
    language: 'ar',
    css: ['css/modern-normalize.css', 'css/style.css'],
  })
  .use(rehypeStringify)
  .process(readSync('README.md'))
  .then(
    file => {
      file.path = 'www/public/index.html';
      writeSync(file);
    },
    err => {
      throw err;
    },
  );

// Building www/public/javascript/dist/bundle.js
exec('cd www/public && npm run build').addListener('error', err => {
  throw err;
});

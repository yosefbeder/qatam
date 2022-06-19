import nodeResolve from '@rollup/plugin-node-resolve';

export default {
  input: 'public/javascript/src/index.js',
  output: {
    file: 'public/javascript/dist/bundle.js',
    format: 'es',
  },
  plugins: [nodeResolve()],
};

import nodeResolve from '@rollup/plugin-node-resolve';

export default {
  input: 'javascript/src/index.js',
  output: {
    file: 'javascript/dist/bundle.js',
    format: 'es',
  },
  plugins: [nodeResolve()],
};

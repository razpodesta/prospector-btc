//@ts-check
const { composePlugins, withNx } = require('@nx/next');

/**
 * @type {import('@nx/next/plugins/with-nx').WithNxOptions}
 **/
const nextConfig = {
  nx: {
    // La propiedad svgr ha sido eliminada en Nx 19+
  },
  // ACTIVACIÓN DE MODO DOCKER OPTIMIZADO
  output: 'standalone',
  // Variables de entorno expuestas al navegador
  env: {
    NEXT_PUBLIC_APP_VERSION: process.env.npm_package_version,
  },
  // Optimización de imágenes
  images: {
    unoptimized: true, // Recomendado para contenedores simples sin CDN
  }
};

const plugins = [
  withNx,
];

module.exports = composePlugins(...plugins)(nextConfig);

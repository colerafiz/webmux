const fs = require('fs');
const path = require('path');

// Simple SVG icon generator for WebMux
const createIcon = (size) => {
  const svg = `<svg width="${size}" height="${size}" viewBox="0 0 ${size} ${size}" xmlns="http://www.w3.org/2000/svg">
  <!-- Background -->
  <rect width="${size}" height="${size}" fill="#0a0a0a" rx="${size * 0.15}"/>
  
  <!-- Terminal prompt -->
  <text x="${size * 0.15}" y="${size * 0.5}" fill="#10b981" font-family="monospace" font-size="${size * 0.25}" font-weight="bold">$_</text>
  
  <!-- Terminal cursor -->
  <rect x="${size * 0.55}" y="${size * 0.35}" width="${size * 0.2}" height="${size * 0.05}" fill="#10b981">
    <animate attributeName="opacity" values="1;0;1" dur="1s" repeatCount="indefinite"/>
  </rect>
</svg>`;
  
  return svg;
};

// Create public directory if it doesn't exist
const publicDir = path.join(__dirname, 'public');
if (!fs.existsSync(publicDir)) {
  fs.mkdirSync(publicDir);
}

// Generate icons
const sizes = [
  { name: 'icon-192.png', size: 192 },
  { name: 'icon-512.png', size: 512 },
  { name: 'apple-touch-icon.png', size: 180 }
];

console.log('Generating icon SVG files...');

sizes.forEach(({ name, size }) => {
  const svgContent = createIcon(size);
  const svgPath = path.join(publicDir, name.replace('.png', '.svg'));
  fs.writeFileSync(svgPath, svgContent);
  console.log(`Created ${svgPath}`);
});

console.log('\nIcon SVG files created successfully!');
console.log('Note: These are SVG files. For production, you should convert them to PNG.');
console.log('You can use online converters or tools like ImageMagick/Inkscape.')
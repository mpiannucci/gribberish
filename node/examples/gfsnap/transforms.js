

// function invert(d) {
//   const shared = {};

//   let p = {
//     type: "Polygon",
//     coordinates: d3.merge(d.coordinates.map(polygon => {
//       return polygon.map(ring => {
//         return ring.map(point => {
//           return [bbox[0] + (point[0] / cols * lngRange), bbox[1] + (point[1] / rows * latRange)];
//         }).reverse();
//       });
//     }))
//   };

//   // Record the y-intersections with the antimeridian.
//   p.coordinates.forEach(ring => {
//     ring.forEach(p => {
//       if (p[0] === -180) shared[p[1]] |= 1;
//       else if (p[0] === 180) shared[p[1]] |= 2;
//     });
//   });

//   // Offset any unshared antimeridian points to prevent their stitching.
//   p.coordinates.forEach(ring => {
//     ring.forEach(p => {
//       if ((p[0] === -180 || p[0] === 180) && shared[p[1]] !== 3) {
//         p[0] = p[0] === -180 ? -179.9995 : 179.9995;
//       }
//     });
//   });

//   p = d3.geoStitch(p);

//   // If the MultiPolygon is empty, treat it as the Sphere.
//   return p.coordinates.length
//     ? { type: "Polygon", coordinates: p.coordinates }
//     : { type: "Sphere" };
// }
module.exports = {
  hooks: {
    beforePacking(pkg) {
      // Remove development-only fields
      delete pkg.devDependencies
      delete pkg.scripts
      // Add publication metadata
      pkg.publishedAt = new Date().toISOString()
      return pkg
    },
  },
}

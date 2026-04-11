import { readFileSync, writeFileSync } from 'node:fs'
import { resolve } from 'node:path'

const root = resolve(new URL('..', import.meta.url).pathname)
const inputPath = resolve(root, 'tauri.conf.json')
const outputArg = process.argv[2] || 'tauri.mobile.generated.conf.json'
const outputPath = resolve(root, outputArg)
const metadataPath = resolve(root, 'mobile-version.generated.json')
const cargoTomlPath = resolve(root, 'Cargo.toml')

function readCargoVersion() {
  const cargoToml = readFileSync(cargoTomlPath, 'utf8')
  const match = cargoToml.match(/^version\s*=\s*"([^"]+)"\s*$/m)
  if (!match) {
    throw new Error('missing package version in Cargo.toml')
  }
  return match[1]
}

function buildVersionCode(version) {
  const match = version.match(/^(\d+)\.(\d+)\.(\d+)/)
  if (!match) {
    return null
  }
  return Number(match[1]) * 1_000_000 + Number(match[2]) * 1_000 + Number(match[3])
}

const semverVersion = process.env.IKB_MOBILE_SEMVER || readCargoVersion()
const versionCode = Number(process.env.IKB_MOBILE_VERSION_CODE || buildVersionCode(semverVersion))
const builtAt = process.env.IKB_MOBILE_BUILT_AT || new Date().toISOString()
const config = JSON.parse(readFileSync(inputPath, 'utf8'))

config.version = semverVersion
config.bundle = config.bundle || {}
config.bundle.icon = ['icons/icon.png', 'icons/32x32.png', 'icons/128x128.png', 'icons/128x128@2x.png', 'icons/icon.ico', 'icons/icon.icns']
config.bundle.android = config.bundle.android || {}
config.bundle.android.versionCode = versionCode
config.bundle.iOS = config.bundle.iOS || {}
config.bundle.iOS.bundleVersion = String(versionCode)

if (process.env.IKB_SKIP_BEFORE_BUILD === '1') {
  config.build = config.build || {}
  config.build.beforeBuildCommand = ''
}

writeFileSync(outputPath, JSON.stringify(config, null, 2) + '\n')
writeFileSync(metadataPath, JSON.stringify({ semverVersion, versionCode, builtAt }, null, 2) + '\n')
process.stdout.write(`${semverVersion}\n`)

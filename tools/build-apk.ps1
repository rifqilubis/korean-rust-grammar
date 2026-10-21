# Bangun APK Android untuk korean-rust-grammar (tanpa Gradle, tanpa Java/Kotlin
# di dalam APK — hanya Rust + Activity native milik eframe).
#
#   powershell -ExecutionPolicy Bypass -File tools/build-apk.ps1
#   powershell -ExecutionPolicy Bypass -File tools/build-apk.ps1 -Debug
#   powershell -ExecutionPolicy Bypass -File tools/build-apk.ps1 -Target aarch64-linux-android
#
# Yang dilakukan skrip ini:
#   1. mencari Android SDK, NDK, dan JDK di mesin ini,
#   2. menulis android/.cargo/config.toml sesuai lokasi NDK (berkas ini
#      dihasilkan otomatis dan tidak perlu di-commit),
#   3. memasang target Rust yang belum ada,
#   4. menjalankan `cargo apk build` di folder android/.
#
# Hasil: android/target/release/apk/korean-rust-grammar.apk

param(
    # Build tanpa optimasi (lebih cepat, APK jauh lebih besar).
    [switch]$Debug,
    # Hanya satu ABI, mis. -Target aarch64-linux-android.
    [string]$Target
)

$ErrorActionPreference = 'Stop'

$root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)
$crate = Join-Path $root 'android'

function Find-Dir([string[]]$candidates) {
    foreach ($c in $candidates) {
        if ($c -and (Test-Path $c)) { return (Resolve-Path $c).Path }
    }
    return $null
}

# ------------------------------------------------------------------ Android SDK
$sdk = Find-Dir @(
    $env:ANDROID_HOME,
    $env:ANDROID_SDK_ROOT,
    (Join-Path $env:LOCALAPPDATA 'Android\Sdk'),
    'C:\Android\Sdk'
)
if (-not $sdk) { throw 'Android SDK tidak ditemukan. Set ANDROID_HOME.' }
# `cargo apk` membaca variabel ini dari lingkungan proses, bukan dari parameter.
$env:ANDROID_HOME = $sdk
Write-Host "SDK      : $sdk"

# ------------------------------------------------------------------ Android NDK
$ndk = $null
if ($env:ANDROID_NDK_HOME -and (Test-Path $env:ANDROID_NDK_HOME)) {
    $ndk = (Resolve-Path $env:ANDROID_NDK_HOME).Path
}
if (-not $ndk) {
    $ndkRoot = Join-Path $sdk 'ndk'
    if (Test-Path $ndkRoot) {
        $latest = Get-ChildItem $ndkRoot -Directory |
            Sort-Object { [version]($_.Name -replace '[^0-9.].*$', '') } -Descending |
            Select-Object -First 1
        if ($latest) { $ndk = $latest.FullName }
    }
}
if (-not $ndk) { throw 'Android NDK tidak ditemukan. Set ANDROID_NDK_HOME.' }
$env:ANDROID_NDK_HOME = $ndk
Write-Host "NDK      : $ndk"

# --------------------------------------------------------------------------- JDK
if (-not $env:JAVA_HOME) { throw 'JAVA_HOME kosong; `cargo apk` membutuhkannya.' }
Write-Host "JAVA_HOME: $env:JAVA_HOME"

# --------------------------------------------------------------- berkas config
$bin = Join-Path $ndk 'toolchains\llvm\prebuilt\windows-x86_64\bin'
if (-not (Test-Path $bin)) {
    $bin = Join-Path $ndk 'toolchains\llvm\prebuilt\windows-x86_64\bin'
}
if (-not (Test-Path $bin)) { throw "Toolchain LLVM NDK tidak ditemukan di $bin" }

# minSdk 24 harus sama dengan `min_sdk_version` di android/Cargo.toml.
$minSdk = 24
$targets = [ordered]@{
    'aarch64-linux-android'    = "aarch64-linux-android$minSdk-clang.cmd"
    'armv7-linux-androideabi'  = "armv7a-linux-androideabi$minSdk-clang.cmd"
    'x86_64-linux-android'     = "x86_64-linux-android$minSdk-clang.cmd"
    'i686-linux-android'       = "i686-linux-android$minSdk-clang.cmd"
}

$configDir = Join-Path $crate '.cargo'
New-Item -ItemType Directory -Force -Path $configDir | Out-Null

$lines = @(
    '# Konfigurasi cargo untuk build Android (Rust murni, tanpa Gradle).',
    '# Dihasilkan otomatis oleh tools/build-apk.ps1 dari ANDROID_NDK_HOME,',
    '# jadi path mesin tidak ditulis manual di sini. Jangan di-commit.',
    ''
)
foreach ($name in $targets.Keys) {
    $linker = (Join-Path $bin $targets[$name]).Replace('\', '/')
    $lines += "[target.$name]"
    $lines += "linker = `"$linker`""
    # Halaman memori 16 KiB: syarat Play Store untuk aplikasi 64-bit.
    $lines += 'rustflags = ["-C", "link-arg=-Wl,-z,max-page-size=16384"]'
    $lines += ''
}
Set-Content -Path (Join-Path $configDir 'config.toml') -Value $lines -Encoding utf8
Write-Host "config   : $(Join-Path $configDir 'config.toml')"

# ------------------------------------------------------------------- Rust target
$installed = rustup target list --installed
foreach ($name in $targets.Keys) {
    if ($installed -notcontains $name) {
        Write-Host "memasang target $name ..."
        rustup target add $name
    }
}

# ------------------------------------------------------------------- kata sandi
# `cargo apk` membaca kunci rilis dari variabel lingkungan ini; tanpa keduanya
# ia menolak menandatangani APK release.
$propsPath = Join-Path $root '.keystore\keystore.properties'
$storePath = Join-Path $root '.keystore\korean-grammar.jks'
if ((Test-Path $propsPath) -and (Test-Path $storePath)) {
    $cfg = @{}
    Get-Content $propsPath | Where-Object { $_ -match '^\s*[^#].*=' } | ForEach-Object {
        $k, $v = $_ -split '=', 2
        $cfg[$k.Trim()] = $v.Trim()
    }
    if (-not $cfg['storePassword']) { throw "storePassword tidak ada di $propsPath" }
    $env:CARGO_APK_RELEASE_KEYSTORE = $storePath
    $env:CARGO_APK_RELEASE_KEYSTORE_PASSWORD = $cfg['storePassword']
    Write-Host "tanda tangan: $storePath"
} else {
    Write-Host 'tanda tangan: dilewati (keystore tidak ada) — APK tidak bisa dipasang sebagai pembaruan'
}

# ------------------------------------------------------------------------ build
$cargoArgs = @('apk', 'build', '--lib')
if (-not $Debug) { $cargoArgs += '--release' }
if ($Target) { $cargoArgs += @('--target', $Target) }

Push-Location $crate
try {
    Write-Host "menjalankan: cargo $($cargoArgs -join ' ')"
    # PowerShell menganggap setiap baris stderr dari program native sebagai
    # error; cargo menulis progres build ke stderr, jadi matikan sementara.
    $previous = $ErrorActionPreference
    $ErrorActionPreference = 'Continue'
    try {
        & cargo @cargoArgs
        $code = $LASTEXITCODE
    }
    finally {
        $ErrorActionPreference = $previous
    }
    if ($code -ne 0) { throw "cargo apk gagal (kode $code)" }
}
finally {
    Pop-Location
}

# ----------------------------------------------------------------------- hasil
$profile = if ($Debug) { 'debug' } else { 'release' }
$apkDir = Join-Path $crate "target\$profile\apk"
if (Test-Path $apkDir) {
    Get-ChildItem $apkDir -Filter *.apk | ForEach-Object {
        Write-Host ''
        Write-Host "APK: $($_.FullName)"
        Write-Host ("ukuran: {0:N1} MB" -f ($_.Length / 1MB))
    }
}

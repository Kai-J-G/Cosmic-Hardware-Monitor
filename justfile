name := 'cosmic-ext-hardware-monitor'
export APPID := 'io.github.kaijg.CosmicHardwareMonitor'

prefix := env_var_or_default("PREFIX", env_var('HOME') / '.local')
bin_dir := prefix / 'bin'
app_dir := prefix / 'share' / 'applications'
metainfo_dir := prefix / 'share' / 'metainfo'
icon_dir := prefix / 'share' / 'icons' / 'hicolor' / 'scalable' / 'apps'

bin_src := 'target' / 'release' / name

default: build-release

build-release:
    cargo build --release

build-debug:
    cargo build

run:
    RUST_BACKTRACE=1 cargo run --release

install: build-release
    install -Dm0755 {{bin_src}} {{bin_dir}}/{{name}}
    install -Dm0644 data/{{APPID}}.desktop {{app_dir}}/{{APPID}}.desktop
    sed -i 's|^Exec=.*|Exec={{bin_dir}}/{{name}}|' {{app_dir}}/{{APPID}}.desktop
    install -Dm0644 data/{{APPID}}.metainfo.xml {{metainfo_dir}}/{{APPID}}.metainfo.xml
    install -Dm0644 data/icons/{{APPID}}-symbolic.svg {{icon_dir}}/{{APPID}}-symbolic.svg
    install -Dm0644 data/icons/{{APPID}}.svg {{icon_dir}}/{{APPID}}.svg
    @echo "Installed Cosmic Hardware Monitor to {{bin_dir}}/{{name}}"
    @echo "Add it via: COSMIC Settings -> Desktop -> Panel -> Applets -> Add Applet"

uninstall:
    rm -f {{bin_dir}}/{{name}}
    rm -f {{app_dir}}/{{APPID}}.desktop
    rm -f {{metainfo_dir}}/{{APPID}}.metainfo.xml
    rm -f {{icon_dir}}/{{APPID}}-symbolic.svg
    rm -f {{icon_dir}}/{{APPID}}.svg
    @echo "Uninstalled Cosmic Hardware Monitor from {{prefix}}"

check:
    cargo check
    cargo test

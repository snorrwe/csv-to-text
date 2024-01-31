dev: tailwind
    trunk serve

fmt:
    cargo fmt
    leptosfmt **/*rs

tailwind:
    tailwindcss -i style/main.css -o style/output.css

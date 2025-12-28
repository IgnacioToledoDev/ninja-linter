🥷 Ninja Linter

CLI en Rust para ejecutar PHP CS Fixer automáticamente sobre archivos .php modificados dentro de un proyecto que corre en Docker.

Pensado para usarse como herramienta de desarrollo diaria.

📦 Requisitos

- Git
- Docker
- Composer dentro del contenedor
- Proyecto PHP con composer cs:fix

🚀 Instalación (recomendada)
Linux /  macOS
```bash
$ curl -fsSL https://github.com/IgnacioToledoDev/ninja-linter/releases/latest/download/install.sh | bash
```


Esto hará:

- Descargar el binario correcto para tu sistema

- Instalarlo en /usr/local/bin

- Requerir sudo si es necesario

Luego puedes ejecutar:

```bash
$ ninja-linter
```

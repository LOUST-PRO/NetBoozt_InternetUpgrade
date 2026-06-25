# NetBoozt - Python Platform

> Versión Python de NetBoozt usando CustomTkinter para la interfaz gráfica.

---

## 📦 Ejecutables

| Ejecutable | Descripción | Uso |
|------------|-------------|-----|
| **NetBoozt_GUI.exe** | Interfaz gráfica con system tray | Usuarios regulares |
| **NetBoozt_CLI.exe** | Línea de comandos interactiva | Usuarios avanzados / servidores |

### NetBoozt_GUI.exe
- Ventana principal con CustomTkinter
- Icono en bandeja del sistema (junto al reloj)
- Monitoreo en tiempo real
- DNS Intelligence con análisis paralelo
- Minimiza a tray al cerrar

### NetBoozt_CLI.exe
- Menú interactivo en terminal
- Diagnósticos de red (4 fases)
- Benchmark de DNS
- Visor de Windows Event Log
- Sin dependencias gráficas

---

## 🚀 Inicio Rápido

### Desarrollo

```powershell
# Ejecutar GUI
python run_modern.py

# Ejecutar CLI
python netboozt_cli.py
```

### Compilar

```powershell
# Desde la raíz del proyecto
.\scripts\build_nuitka.ps1                # Ambos ejecutables
.\scripts\build_nuitka.ps1 -Target GUI    # Solo GUI
.\scripts\build_nuitka.ps1 -Target CLI    # Solo CLI
```

---

## 📁 Estructura de Archivos

```
platforms/python/
├── NetBoozt_GUI.py      # Entry point GUI (→ run_modern.py)
├── run_modern.py        # Launcher principal con splash + tray
├── netboozt_cli.py      # CLI interactivo completo
├── netboozt.spec        # Spec PyInstaller (legacy)
├── requirements.txt     # Dependencias Python
│
├── src/                 # Código fuente
│   ├── core/            # Lógica central
│   ├── gui/             # Componentes CustomTkinter
│   │   ├── modern_window.py      # Ventana principal
│   │   ├── dashboard.py          # Panel métricas
│   │   ├── system_tray.py        # Icono bandeja ⭐ NEW
│   │   ├── dns_intelligence_tab.py # Tab DNS ⭐ NEW
│   │   └── ...
│   ├── monitoring/      # Monitoreo de red
│   │   ├── realtime_monitor.py   # Métricas tiempo real
│   │   ├── dns_intelligence.py   # Análisis DNS ⭐ NEW
│   │   ├── network_diagnostics.py # 4-fases
│   │   ├── windows_events.py     # Event Log
│   │   └── ...
│   ├── optimizations/   # TCP/IP optimizations
│   ├── storage/         # Persistencia (TinyDB)
│   └── utils/           # Utilidades
│
├── assets/              # Imágenes, iconos
├── logs/                # Archivos de log
└── tests/               # Tests unitarios
```

---

## 📋 Dependencias

### Principales

| Paquete | Versión | Propósito |
|---------|---------|-----------|
| customtkinter | ≥5.2.0 | GUI moderna |
| pillow | ≥10.0.0 | Procesamiento de imágenes |
| psutil | ≥5.9.0 | Métricas del sistema |
| pystray | ≥0.19.0 | System tray icon |
| tinydb | ≥4.8.0 | Base de datos local |

### Compilación

| Paquete | Versión | Propósito |
|---------|---------|-----------|
| nuitka | ≥1.8.0 | Compilador Python→C |
| ordered-set | ≥4.1.0 | Dependencia Nuitka |
| zstandard | ≥0.21.0 | Compresión Nuitka |

### Instalar Todo

```powershell
pip install -r requirements.txt
```

---

## 🔧 Módulos Clave

### DNS Intelligence (`src/monitoring/dns_intelligence.py`)

```python
from src.monitoring import get_dns_intelligence

intel = get_dns_intelligence()
intel.start_monitoring()

# Obtener mejor DNS
best = intel.get_best_dns()
print(f"Mejor: {best.name} - {best.score} pts")

# Ranking completo
for rank, (server, metrics) in enumerate(intel.get_ranking(), 1):
    print(f"#{rank} {metrics.name}: ping={metrics.avg_ping}ms")
```

### System Tray (`src/gui/system_tray.py`)

```python
from src.gui import SystemTrayIcon

tray = SystemTrayIcon(
    on_show=show_window,
    on_quit=quit_app
)
tray.run()  # En thread separado

# Cambiar estado
tray.set_status("warning", "DNS lento")
```

---

## 🧪 Testing

```powershell
# Todos los tests
python -m pytest tests/

# Test específico
python -m pytest tests/test_dns_intelligence.py -v
```

---

## 📄 Licencia

- **Código abierto**: Apache-2.0 License
- **Versión LOUST**: Propietaria

---

<div align="center">

**Made with ❤️ by [LOUST](https://www.loust.pro)**

</div>

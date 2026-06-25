# Contributing to NetBoozt 🚀

¡Gracias por considerar contribuir a NetBoozt! Este proyecto busca democratizar las optimizaciones de red avanzadas.

## 🎯 Formas de Contribuir

### 1. Reportar Bugs
- Usa la plantilla de issues en GitHub
- Incluye pasos para reproducir
- Especifica sistema operativo y versión
- Adjunta logs si es posible

### 2. Proponer Optimizaciones
- Documenta el beneficio esperado
- Incluye benchmarks/evidencia
- Explica configuración de registro (Windows/Linux)
- Proporciona rollback seguro

### 3. Mejorar Documentación
- Correcciones de typos
- Ejemplos adicionales
- Traducciones
- Diagramas mejorados

### 4. Desarrollar Features
- Discute primero en issues
- Sigue las convenciones de código
- Incluye tests
- Actualiza documentación

## 🔧 Setup de Desarrollo

### Windows
```powershell
# Clone repository
git clone git@github.com:LOUST-PRO/NetBoozt_InternetUpgrade.git
cd NetBoozt_InternetUpgrade

# Create virtual environment
python -m venv venv --copies
.\venv\Scripts\Activate.ps1

# Install dependencies
pip install -r windows/requirements.txt
pip install -r requirements-dev.txt

# Run tests
pytest windows/tests/

# Run GUI
python windows/run.py
```

### Linux
```bash
# Clone repository
git clone git@github.com:LOUST-PRO/NetBoozt_InternetUpgrade.git
cd NetBoozt_InternetUpgrade

# Create virtual environment
python3 -m venv venv
source venv/bin/activate

# Install dependencies
pip install -r linux/requirements.txt
pip install -r requirements-dev.txt

# Run tests
pytest linux/tests/

# Run CLI
python linux/netboozt.py --help
```

## 📝 Convenciones de Código

### Python
- **Style**: PEP 8 (usa `black` formatter)
- **Imports**: `isort` para ordenar
- **Docstrings**: Google style
- **Type hints**: Obligatorios para funciones públicas

Ejemplo:
```python
def apply_optimization(
    name: str, 
    value: int, 
    backup: bool = True
) -> tuple[bool, str]:
    """Apply network optimization to system.
    
    Args:
        name: Optimization name (e.g., 'TcpWindowSize')
        value: Registry value to set
        backup: Whether to backup current value
        
    Returns:
        Tuple of (success: bool, message: str)
        
    Raises:
        PermissionError: If not running as admin
    """
    pass
```

### Commits
Usa **Conventional Commits**:
- `feat:` Nueva funcionalidad
- `fix:` Corrección de bug
- `docs:` Cambios en documentación
- `style:` Formato (no afecta código)
- `refactor:` Refactorización
- `test:` Tests
- `chore:` Tareas de mantenimiento

Ejemplo:
```
feat(windows): Add BBR congestion control support

- Implement BBR algorithm toggle
- Add documentation for BBR vs NewReno
- Include benchmarks showing 20% throughput improvement

Closes #42
```

### Branches
- `main`: Producción estable
- `develop`: Desarrollo activo
- `feature/*`: Nuevas funcionalidades
- `bugfix/*`: Correcciones
- `docs/*`: Documentación

## 🧪 Testing

### Requisitos
- **Coverage mínimo**: 80%
- Tests unitarios para funciones críticas
- Tests de integración para optimizaciones
- Mock de registry/sysctl para tests seguros

### Ejecutar Tests
```powershell
# Windows
pytest windows/tests/ --cov=windows/src --cov-report=html

# Linux
pytest linux/tests/ --cov=linux/src --cov-report=html
```

## 📚 Documentación

### Estructura de Optimización
Cada optimización debe tener:
```markdown
# [Nombre de Optimización]

## 📊 Descripción
[Explicación breve y técnica]

## 🎯 Beneficios
- Beneficio 1
- Beneficio 2

## ⚙️ Configuración
[Tabla con registro/sysctl]

## 📈 Benchmarks
[Resultados antes/después]

## 🔄 Rollback
[Comandos para revertir]

## ⚠️ Advertencias
[Precauciones]
```

### Mermaid Diagrams
- Usa `flowchart TB` para procesos
- Usa `graph LR` para arquitecturas
- Colores consistentes (ver `/docs/diagrams/architecture.md`)

## 🚀 Pull Request Process

1. **Fork** el repositorio
2. **Crea branch** desde `develop`
   ```bash
   git checkout -b feature/mi-nueva-funcionalidad
   ```
3. **Haz commits** siguiendo convenciones
4. **Ejecuta tests** y verifica coverage
5. **Actualiza docs** si es necesario
6. **Push** a tu fork
   ```bash
   git push origin feature/mi-nueva-funcionalidad
   ```
7. **Abre PR** hacia `develop`
8. **Responde a reviews** de manera constructiva

### Checklist de PR
- [ ] Tests pasan (`pytest`)
- [ ] Coverage ≥ 80% (`pytest --cov`)
- [ ] Code formateado (`black .`)
- [ ] Imports ordenados (`isort .`)
- [ ] Docs actualizadas (si aplica)
- [ ] CHANGELOG.md actualizado
- [ ] PR description clara

## 🌍 Código de Conducta

### Nuestro Compromiso
NetBoozt es un proyecto inclusivo:
- ✅ Respeto mutuo
- ✅ Colaboración constructiva
- ✅ Aceptación de críticas
- ❌ Lenguaje ofensivo
- ❌ Ataques personales
- ❌ Trolling/spam

### Reportar Comportamiento
Email: opensource@loust.pro

## 📧 Contacto

- **Website**: www.loust.pro
- **Email**: opensource@loust.pro
- **GitHub Issues**: [NetBoozt Issues](https://github.com/LOUST-PRO/NetBoozt_InternetUpgrade/issues)
- **Discussions**: [NetBoozt Discussions](https://github.com/LOUST-PRO/NetBoozt_InternetUpgrade/discussions)

## 🎖️ Reconocimientos

Todos los colaboradores serán listados en:
- `README.md` (sección Contributors)
- GitHub Contributors page
- Release notes

¡Gracias por hacer NetBoozt mejor! 🚀💙

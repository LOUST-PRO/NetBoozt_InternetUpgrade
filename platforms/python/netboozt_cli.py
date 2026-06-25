#!/usr/bin/env python3
"""
NetBoozt CLI - Interactive Build & Development Tool
Inspired by Next.js/Astro CLI experience

By LOUST (www.loust.pro)
"""

import sys
import subprocess
import os
from pathlib import Path
import shutil
import ctypes
import time

# Habilitar ANSI colors en Windows CMD
def enable_ansi_colors():
    """Habilitar códigos ANSI en Windows CMD/PowerShell"""
    if sys.platform == 'win32':
        try:
            # Habilitar ANSI escape sequences en Windows 10+
            kernel32 = ctypes.windll.kernel32
            kernel32.SetConsoleMode(kernel32.GetStdHandle(-11), 7)
        except:
            pass

enable_ansi_colors()

# Colores ANSI
class Colors:
    PRIMARY = '\033[96m'      # Cyan (similar a LOUST green)
    SUCCESS = '\033[92m'      # Green
    WARNING = '\033[93m'      # Yellow
    ERROR = '\033[91m'        # Red
    INFO = '\033[94m'         # Blue
    BOLD = '\033[1m'
    DIM = '\033[2m'
    RESET = '\033[0m'

def is_admin():
    """Verificar si el script se está ejecutando como administrador"""
    try:
        return ctypes.windll.shell32.IsUserAnAdmin()
    except:
        return False

def run_as_admin():
    """Reiniciar el script con permisos de administrador usando PowerShell"""
    try:
        script = os.path.abspath(sys.argv[0])
        
        # Usar PowerShell Start-Process con -Verb RunAs
        # Mantener en PowerShell (no cambiar a cmd)
        cmd = f'powershell -Command "Start-Process powershell -ArgumentList \'-NoProfile\', \'-ExecutionPolicy\', \'Bypass\', \'-Command\', \'cd \\\"{os.path.dirname(script)}\\\"; python \\\"{script}\\\" --no-welcome; Read-Host \\\"Press Enter to exit\\\"\' -Verb RunAs"'
        
        subprocess.Popen(cmd, shell=True)
        return True
    except Exception as e:
        print(f"{Colors.ERROR}✗{Colors.RESET} Error requesting admin: {e}")
        return False

def print_banner():
    """Imprimir banner de NetBoozt con estado admin"""
    admin_indicator = ""
    if is_admin():
        admin_indicator = f"\n{Colors.SUCCESS}⚡ Administrator Mode: ACTIVE{Colors.RESET}"
    else:
        admin_indicator = f"\n{Colors.DIM}⚠ Standard Mode (some features require elevation){Colors.RESET}"
    
    banner = f"""
{Colors.PRIMARY}{Colors.BOLD}
╔══════════════════════════════════════════════════════════════╗
║                  NetBoozt Development CLI                    ║
║                  Network Performance Optimizer               ║
║                  By LOUST (www.loust.pro)                    ║
╚══════════════════════════════════════════════════════════════╝
{Colors.RESET}{admin_indicator}
"""
    print(banner)

def print_welcome():
    """Pantalla de bienvenida animada con explicación de permisos"""
    # Limpiar pantalla
    os.system('cls' if os.name == 'nt' else 'clear')
    
    # Banner
    print(f"""
{Colors.PRIMARY}{Colors.BOLD}
    ╔════════════════════════════════════════════════════════╗
    ║                                                        ║
    ║              🚀  Welcome to NetBoozt CLI  🚀           ║
    ║                                                        ║
    ║          Network Performance Optimization Tool        ║
    ║                    by LOUST.pro                       ║
    ║                                                        ║
    ╚════════════════════════════════════════════════════════╝
{Colors.RESET}
""")
    
    time.sleep(0.3)
    
    # Información sobre permisos
    admin_status = is_admin()
    
    if admin_status:
        print(f"{Colors.SUCCESS}✓{Colors.RESET} {Colors.BOLD}Administrator Mode: {Colors.SUCCESS}ENABLED{Colors.RESET}")
        print(f"{Colors.DIM}  Full access to all CLI features{Colors.RESET}\n")
    else:
        print(f"{Colors.WARNING}⚠{Colors.RESET} {Colors.BOLD}Administrator Mode: {Colors.WARNING}DISABLED{Colors.RESET}")
        print(f"{Colors.DIM}  Some features will prompt for elevation{Colors.RESET}\n")
    
    # Información importante para desarrollo
    print(f"{Colors.INFO}💡{Colors.RESET} {Colors.BOLD}Quick Start Guide:{Colors.RESET}\n")
    print(f"  {Colors.SUCCESS}For End Users:{Colors.RESET}")
    print(f"  {Colors.DIM}• Use option{Colors.RESET} {Colors.PRIMARY}4 (Run){Colors.RESET} {Colors.DIM}to launch the pre-compiled .exe{Colors.RESET}")
    print(f"  {Colors.DIM}• No compilation needed - ready to use!{Colors.RESET}\n")
    
    print(f"  {Colors.WARNING}For Developers:{Colors.RESET}")
    print(f"  {Colors.DIM}• Use{Colors.RESET} {Colors.PRIMARY}1 (Build){Colors.RESET} {Colors.DIM}or{Colors.RESET} {Colors.PRIMARY}3 (Rebuild){Colors.RESET} {Colors.DIM}to compile from source{Colors.RESET}")
    print(f"  {Colors.DIM}• {Colors.WARNING}Important:{Colors.RESET} {Colors.DIM}Add this folder to your antivirus exclusions{Colors.RESET}")
    print(f"    {Colors.DIM}→ PyInstaller compilation may trigger false positives{Colors.RESET}")
    print(f"    {Colors.DIM}→ This is normal for Python-to-exe compilers{Colors.RESET}")
    print(f"    {Colors.DIM}→ See:{Colors.RESET} {Colors.INFO}https://pyinstaller.org/en/stable/when-things-go-wrong.html{Colors.RESET}\n")
    
    print(f"{Colors.INFO}ℹ{Colors.RESET} {Colors.BOLD}Why Administrator Access?{Colors.RESET}\n")
    print(f"{Colors.DIM}  Required for specific operations:{Colors.RESET}")
    print(f"  {Colors.PRIMARY}•{Colors.RESET} Delete locked files in {Colors.DIM}dist/{Colors.RESET} and {Colors.DIM}build/{Colors.RESET} folders")
    print(f"  {Colors.PRIMARY}•{Colors.RESET} Terminate {Colors.DIM}NetBoozt.exe{Colors.RESET} background processes")
    print(f"  {Colors.PRIMARY}•{Colors.RESET} Copy compiled .exe to {Colors.DIM}Desktop{Colors.RESET} and repo root\n")
    
    print(f"{Colors.SUCCESS}✓{Colors.RESET} {Colors.BOLD}Safe & Transparent:{Colors.RESET}")
    print(f"  {Colors.DIM}• User-space only - no kernel operations{Colors.RESET}")
    print(f"  {Colors.DIM}• Only touches NetBoozt project files{Colors.RESET}")
    print(f"  {Colors.DIM}• You approve each elevation request{Colors.RESET}")
    print(f"  {Colors.DIM}• All actions logged to logs/ directory{Colors.RESET}\n")
    
    time.sleep(0.3)
    
    # Opciones de navegación
    print(f"{Colors.BOLD}Quick Navigation:{Colors.RESET}\n")
    print(f"  {Colors.PRIMARY}1{Colors.RESET} › Continue to Main Menu")
    print(f"  {Colors.PRIMARY}2{Colors.RESET} › View README.md")
    print(f"  {Colors.PRIMARY}3{Colors.RESET} › Open Documentation")
    print(f"  {Colors.PRIMARY}q{Colors.RESET} › {Colors.DIM}Quit{Colors.RESET}\n")
    
    choice = input(f"{Colors.PRIMARY}>{Colors.RESET} ").strip().lower()
    
    if choice == '2' or choice == 'readme':
        show_readme()
        return print_welcome()  # Volver al welcome
    elif choice == '3' or choice == 'docs':
        show_docs()
        return print_welcome()  # Volver al welcome
    elif choice == 'q' or choice == 'quit':
        print(f"\n{Colors.DIM}Goodbye! 👋{Colors.RESET}\n")
        sys.exit(0)
    
    # Animación de carga
    print(f"\n{Colors.INFO}▶{Colors.RESET} {Colors.BOLD}Loading CLI...{Colors.RESET}\n")
    for i in range(3):
        print(f"{Colors.DIM}  .{Colors.RESET}", end='', flush=True)
        time.sleep(0.2)
    
    print(f" {Colors.SUCCESS}Ready!{Colors.RESET}\n")
    time.sleep(0.3)
    
    # Limpiar pantalla para mostrar menú
    os.system('cls' if os.name == 'nt' else 'clear')

def show_readme():
    """Mostrar README.md"""
    os.system('cls' if os.name == 'nt' else 'clear')
    readme_path = Path("..") / "README.md"
    
    if readme_path.exists():
        print(f"{Colors.PRIMARY}{Colors.BOLD}README.md{Colors.RESET}\n")
        
        # Abrir en notepad
        try:
            subprocess.Popen(['notepad.exe', str(readme_path.absolute())])
            print(f"{Colors.SUCCESS}✓{Colors.RESET} Opened README.md in Notepad\n")
        except:
            print(f"{Colors.ERROR}✗{Colors.RESET} Could not open README\n")
    else:
        print(f"{Colors.WARNING}⚠{Colors.RESET} README.md not found\n")
    
    input(f"{Colors.DIM}Press Enter to go back...{Colors.RESET}")

def show_docs():
    """Abrir documentación"""
    os.system('cls' if os.name == 'nt' else 'clear')
    print(f"{Colors.PRIMARY}{Colors.BOLD}Documentation{Colors.RESET}\n")
    print(f"{Colors.INFO}▶{Colors.RESET} Opening documentation in browser...\n")
    
    import webbrowser
    # Abrir docs en GitHub o link de docs
    webbrowser.open("https://github.com/LOUST-PRO/NetBoozt_InternetUpgrade#readme")
    
    print(f"{Colors.SUCCESS}✓{Colors.RESET} Opened in browser\n")
    input(f"{Colors.DIM}Press Enter to go back...{Colors.RESET}")

def print_menu():
    """Mostrar menú principal"""
    menu = f"""
{Colors.BOLD}What would you like to do?{Colors.RESET}

  {Colors.PRIMARY}1{Colors.RESET} › {Colors.BOLD}Build{Colors.RESET}           Compile from source (auto-copy to local repo)
  {Colors.PRIMARY}2{Colors.RESET} › {Colors.BOLD}Dev{Colors.RESET}             Run in development mode
  {Colors.PRIMARY}3{Colors.RESET} › {Colors.BOLD}Rebuild{Colors.RESET}         Clean + Build from scratch
  {Colors.PRIMARY}4{Colors.RESET} › {Colors.BOLD}Run{Colors.RESET}             Execute the pre-compiled .exe
  {Colors.PRIMARY}5{Colors.RESET} › {Colors.BOLD}Deploy{Colors.RESET}         Copy .exe to Desktop
  {Colors.PRIMARY}6{Colors.RESET} › {Colors.BOLD}GitHub{Colors.RESET}         Open repository in browser
  {Colors.PRIMARY}7{Colors.RESET} › {Colors.BOLD}Web{Colors.RESET}            Visit www.loust.pro
  
  {Colors.INFO}--- Network Tools ---{Colors.RESET}
  {Colors.PRIMARY}d{Colors.RESET} › {Colors.BOLD}Diagnose{Colors.RESET}       Full network diagnostic
  {Colors.PRIMARY}n{Colors.RESET} › {Colors.BOLD}DNS Test{Colors.RESET}       Benchmark DNS servers
  {Colors.PRIMARY}w{Colors.RESET} › {Colors.BOLD}Win Events{Colors.RESET}     Show Windows network events
  {Colors.PRIMARY}f{Colors.RESET} › {Colors.BOLD}Fix DNS{Colors.RESET}        Apply optimal DNS settings
  
  {Colors.DIM}--- Other ---{Colors.RESET}
  {Colors.PRIMARY}r{Colors.RESET} › {Colors.DIM}Report{Colors.RESET}         Report bug or feedback
  {Colors.PRIMARY}l{Colors.RESET} › {Colors.DIM}Logs{Colors.RESET}           View latest log file
  {Colors.PRIMARY}t{Colors.RESET} › {Colors.DIM}Tree{Colors.RESET}           View project file tree
  {Colors.PRIMARY}c{Colors.RESET} › {Colors.DIM}Clean{Colors.RESET}          Remove build artifacts
  {Colors.PRIMARY}v{Colors.RESET} › {Colors.DIM}Verify{Colors.RESET}         Check build requirements
  {Colors.PRIMARY}q{Colors.RESET} › {Colors.DIM}Quit{Colors.RESET}

{Colors.DIM}Type the number or letter{Colors.RESET}
"""
    print(menu)

def run_command(cmd, description, cwd=None):
    """Ejecutar comando con output bonito"""
    print(f"\n{Colors.INFO}▶{Colors.RESET} {description}...")
    try:
        result = subprocess.run(
            cmd,
            shell=True,
            cwd=cwd,
            capture_output=False,
            text=True
        )
        if result.returncode == 0:
            print(f"{Colors.SUCCESS}✓{Colors.RESET} {description} completed")
            return True
        else:
            print(f"{Colors.ERROR}✗{Colors.RESET} {description} failed")
            return False
    except Exception as e:
        print(f"{Colors.ERROR}✗{Colors.RESET} Error: {e}")
        return False

def action_build():
    """Compilar NetBoozt a .exe"""
    print(f"\n{Colors.BOLD}Building NetBoozt...{Colors.RESET}\n")
    
    # Advertir si estamos en modo admin
    if is_admin():
        print(f"{Colors.WARNING}⚠ Warning: Running as Administrator{Colors.RESET}")
        print(f"{Colors.DIM}  PyInstaller works best without admin privileges{Colors.RESET}")
        print(f"{Colors.DIM}  This won't affect the build, but is not recommended{Colors.RESET}\n")
        time.sleep(1)
    
    # Verificar primero
    if not run_command("python verify_build.py", "Pre-build verification"):
        print(f"\n{Colors.WARNING}⚠{Colors.RESET} Fix errors before building")
        return False
    
    # Limpiar builds anteriores
    print(f"\n{Colors.INFO}▶{Colors.RESET} Cleaning previous builds...")
    shutil.rmtree("build", ignore_errors=True)
    shutil.rmtree("dist", ignore_errors=True)
    print(f"{Colors.SUCCESS}✓{Colors.RESET} Cleaned")
    
    # Compilar
    print(f"\n{Colors.INFO}▶{Colors.RESET} Compiling with PyInstaller...")
    print(f"{Colors.DIM}This may take 2-3 minutes...{Colors.RESET}\n")
    
    if run_command("python -m PyInstaller netboozt.spec --noconfirm", "Compilation"):
        exe_path = Path("dist/NetBoozt.exe")
        if exe_path.exists():
            size_mb = exe_path.stat().st_size / (1024 * 1024)
            print(f"\n{Colors.SUCCESS}✓ Build successful!{Colors.RESET}")
            print(f"{Colors.INFO}  Size:{Colors.RESET} {size_mb:.2f} MB")
            print(f"{Colors.INFO}  Path:{Colors.RESET} {exe_path.absolute()}")
            
            # Auto-copy al repo (paso 6 automático)
            repo_path = Path("..") / "NetBoozt.exe"
            shutil.copy2(exe_path, repo_path)
            print(f"{Colors.SUCCESS}✓{Colors.RESET} Auto-copied to repo: {repo_path.absolute()}")
            
            return True
    
    return False

def action_dev():
    """Ejecutar en modo desarrollo"""
    print(f"\n{Colors.BOLD}Starting NetBoozt in dev mode...{Colors.RESET}\n")
    run_command("python run_modern.py", "Running app")

def action_verify():
    """Verificar requisitos de build"""
    print(f"\n{Colors.BOLD}Verifying build requirements...{Colors.RESET}\n")
    run_command("python verify_build.py", "Verification")

def do_cleanup(kill_processes=True, ask_for_admin=True):
    """Función central de limpieza reutilizable"""
    # Intentar matar procesos NetBoozt primero
    if kill_processes:
        try:
            result = subprocess.run(
                "taskkill /F /IM NetBoozt.exe 2>$null",
                shell=True,
                capture_output=True
            )
            if result.returncode == 0:
                print(f"{Colors.INFO}▶{Colors.RESET} Closed running NetBoozt instances")
                import time
                time.sleep(0.5)  # Esperar a que se liberen archivos
        except:
            pass
    
    dirs_to_clean = ["build", "dist", "__pycache__"]
    permission_errors = []
    
    for dir_name in dirs_to_clean:
        if Path(dir_name).exists():
            try:
                shutil.rmtree(dir_name)
                print(f"{Colors.SUCCESS}✓{Colors.RESET} Removed {dir_name}/")
            except PermissionError as e:
                permission_errors.append(dir_name)
                print(f"{Colors.WARNING}⚠{Colors.RESET} Cannot remove {dir_name}/ (file locked)")
    
    if permission_errors:
        print(f"\n{Colors.WARNING}⚠ Some files are locked by Windows{Colors.RESET}")
        
        if ask_for_admin and not is_admin():
            print(f"\n{Colors.INFO}This operation requires Administrator privileges.{Colors.RESET}")
            print(f"{Colors.DIM}A UAC prompt will appear - please click 'Yes' to continue.{Colors.RESET}\n")
            
            choice = input(f"{Colors.PRIMARY}Request admin access? (Y/n):{Colors.RESET} ").strip().lower()
            
            if choice == '' or choice == 'y' or choice == 'yes':
                print(f"\n{Colors.INFO}▶{Colors.RESET} Opening new window with admin privileges...")
                print(f"{Colors.DIM}(This window will close and a new one will open){Colors.RESET}\n")
                
                if run_as_admin():
                    input("Press Enter to close this window...")
                    sys.exit(0)
                else:
                    print(f"\n{Colors.ERROR}✗{Colors.RESET} Could not request admin access")
                    print(f"{Colors.INFO}ℹ{Colors.RESET} Try running the .bat file as administrator manually")
            else:
                print(f"\n{Colors.WARNING}⚠{Colors.RESET} Skipping locked files")
        else:
            # Ya tenemos admin pero aún hay errores o no queremos preguntar
            if ask_for_admin:
                print(f"\n{Colors.ERROR}✗{Colors.RESET} Cannot remove files even with admin privileges")
                print(f"{Colors.INFO}ℹ{Colors.RESET} Files may be in use by another process")
        return False
    else:
        print(f"\n{Colors.SUCCESS}✓ Clean complete{Colors.RESET}")
        return True

def action_clean():
    """Limpiar artifacts de build"""
    print(f"\n{Colors.BOLD}Cleaning build artifacts...{Colors.RESET}\n")
    do_cleanup(kill_processes=True, ask_for_admin=True)

def action_rebuild():
    """Limpiar y buildear desde cero"""
    print(f"\n{Colors.BOLD}Rebuild: Clean + Build{Colors.RESET}\n")
    print(f"{Colors.BOLD}Cleaning build artifacts...{Colors.RESET}\n")
    
    # Usar función centralizada sin preguntar por admin (auto-matar procesos)
    if do_cleanup(kill_processes=True, ask_for_admin=False):
        input(f"\n{Colors.DIM}Press Enter to start building...{Colors.RESET}")
        action_build()
    else:
        print(f"\n{Colors.ERROR}✗{Colors.RESET} Clean failed - cannot continue with build")
        print(f"{Colors.INFO}ℹ{Colors.RESET} Close any running NetBoozt.exe instances and try again")

def action_run():
    """Ejecutar el .exe compilado con permisos de administrador"""
    exe_path = Path("dist/NetBoozt.exe")
    if not exe_path.exists():
        print(f"{Colors.ERROR}✗{Colors.RESET} NetBoozt.exe not found. Build first.")
        return
    
    print(f"\n{Colors.INFO}▶{Colors.RESET} Running NetBoozt.exe...")
    print(f"\n{Colors.BOLD}Why Administrator Privileges?{Colors.RESET}")
    print(f"{Colors.DIM}NetBoozt needs elevated permissions for:{Colors.RESET}")
    print(f"{Colors.DIM}  • Network adapter configuration (MTU, DNS, etc.){Colors.RESET}")
    print(f"{Colors.DIM}  • System-level network optimization{Colors.RESET}")
    print(f"{Colors.DIM}  • Windows network stack adjustments{Colors.RESET}")
    print(f"{Colors.DIM}  • Performance registry modifications{Colors.RESET}")
    print(f"\n{Colors.WARNING}⚡{Colors.RESET} You'll see a UAC prompt - click {Colors.SUCCESS}Yes{Colors.RESET} to continue\n")
    
    # Usar PowerShell Start-Process con -Verb RunAs para solicitar admin
    cmd = f'powershell -Command "Start-Process \'{exe_path.absolute()}\' -Verb RunAs"'
    try:
        subprocess.Popen(cmd, shell=True)
        print(f"{Colors.SUCCESS}✓{Colors.RESET} Launched with administrator privileges")
    except Exception as e:
        print(f"{Colors.ERROR}✗{Colors.RESET} Error: {e}")

def action_deploy():
    """Copiar .exe al desktop del usuario"""
    exe_path = Path("dist/NetBoozt.exe")
    if not exe_path.exists():
        print(f"{Colors.ERROR}✗{Colors.RESET} NetBoozt.exe not found. Build first.")
        return
    
    print(f"\n{Colors.BOLD}Deploying to Desktop...{Colors.RESET}\n")
    
    # Copiar al desktop
    desktop = Path.home() / "Desktop" / "NetBoozt.exe"
    shutil.copy2(exe_path, desktop)
    print(f"{Colors.SUCCESS}✓{Colors.RESET} Copied to {desktop}")
    
    print(f"\n{Colors.SUCCESS}✓ Deploy complete{Colors.RESET}")

def action_github():
    """Abrir repo en navegador"""
    import webbrowser
    repo_url = "https://github.com/LOUST-PRO/NetBoozt_InternetUpgrade"
    print(f"\n{Colors.INFO}▶{Colors.RESET} Opening GitHub repo...")
    webbrowser.open(repo_url)
    print(f"{Colors.SUCCESS}✓{Colors.RESET} Opened {repo_url}")

def action_web():
    """Abrir www.loust.pro"""
    import webbrowser
    web_url = "https://www.loust.pro"
    print(f"\n{Colors.INFO}▶{Colors.RESET} Opening LOUST website...")
    webbrowser.open(web_url)
    print(f"{Colors.SUCCESS}✓{Colors.RESET} Opened {web_url}")

def action_logs():
    """Ver el archivo de log más reciente"""
    import datetime
    
    # Buscar logs en múltiples ubicaciones (orden de prioridad)
    possible_log_dirs = [
        Path("dist/logs"),  # Después de compilar (prioridad)
        Path("logs"),  # Desarrollo
        Path("../logs"),  # Si estamos en subdirectorio
        Path("../windows/dist/logs"),  # Desde raíz del repo
    ]
    
    # Buscar en TODAS las ubicaciones y combinar
    all_log_files = []
    seen_names = set()  # Para evitar duplicados
    
    for log_path in possible_log_dirs:
        if log_path.exists() and log_path.is_dir():
            logs_found = list(log_path.glob("netboozt_*.log"))
            for log_file in logs_found:
                # Deduplicar por nombre (solo agregar si no lo hemos visto)
                if log_file.name not in seen_names:
                    all_log_files.append(log_file)
                    seen_names.add(log_file.name)
    
    if not all_log_files:
        print(f"\n{Colors.WARNING}⚠ No log files found{Colors.RESET}")
        print(f"\n{Colors.DIM}Searched in:{Colors.RESET}")
        for log_path in possible_log_dirs:
            exists = "✓" if log_path.exists() else "✗"
            print(f"  {Colors.DIM}{exists} {log_path.absolute()}{Colors.RESET}")
        return
    
    # Ordenar por fecha (más reciente primero)
    all_log_files = sorted(all_log_files, key=lambda f: f.stat().st_mtime, reverse=True)
    latest_log = all_log_files[0]
    
    print(f"\n{Colors.BOLD}📋 Log Files ({len(all_log_files)} found){Colors.RESET}\n")
    
    # Header de tabla
    print(f"{Colors.DIM}{'Status':<8} {'Filename':<40} {'Size':<10} {'Modified':<20}{Colors.RESET}")
    print(f"{Colors.DIM}{'-'*80}{Colors.RESET}")
    
    # Mostrar logs (hasta 10)
    for i, log_file in enumerate(all_log_files[:10]):
        size_kb = log_file.stat().st_size / 1024
        mod_time = datetime.datetime.fromtimestamp(log_file.stat().st_mtime)
        time_str = mod_time.strftime("%Y-%m-%d %H:%M:%S")
        
        if i == 0:
            # El más reciente en verde
            status = f"{Colors.SUCCESS}LATEST{Colors.RESET}"
            filename = f"{Colors.SUCCESS}{log_file.name}{Colors.RESET}"
        else:
            status = f"{Colors.DIM}  --  {Colors.RESET}"
            filename = f"{Colors.DIM}{log_file.name}{Colors.RESET}"
        
        print(f"{status:<20} {filename:<50} {size_kb:>6.1f} KB  {Colors.DIM}{time_str}{Colors.RESET}")
    
    if len(all_log_files) > 10:
        print(f"\n{Colors.DIM}... and {len(all_log_files) - 10} more{Colors.RESET}")
    
    # Abrir el más reciente con notepad
    print(f"\n{Colors.INFO}▶{Colors.RESET} Opening latest log in Notepad...")
    print(f"{Colors.DIM}Location: {latest_log.absolute()}{Colors.RESET}\n")
    
    try:
        subprocess.Popen(['notepad.exe', str(latest_log.absolute())])
        print(f"{Colors.SUCCESS}✓{Colors.RESET} Opened in Notepad")
    except Exception as e:
        print(f"{Colors.ERROR}✗{Colors.RESET} Error opening log: {e}")

def action_tree():
    """Mostrar árbol de archivos del proyecto"""
    print(f"\n{Colors.BOLD}Project File Tree{Colors.RESET}\n")
    
    # Directorios importantes a mostrar
    important_dirs = [
        ("src/", "Source code"),
        ("logs/", "Log files"),
        ("assets/", "Images & resources"),
        ("dist/", "Compiled .exe"),
        ("build/", "Build artifacts"),
    ]
    
    for dir_path, description in important_dirs:
        path = Path(dir_path)
        if path.exists():
            if path.is_dir():
                files = list(path.rglob("*"))
                file_count = len([f for f in files if f.is_file()])
                print(f"{Colors.SUCCESS}✓{Colors.RESET} {dir_path:<20} {Colors.DIM}{description} ({file_count} files){Colors.RESET}")
            else:
                print(f"{Colors.INFO}•{Colors.RESET} {dir_path:<20} {Colors.DIM}{description}{Colors.RESET}")
        else:
            print(f"{Colors.DIM}✗ {dir_path:<20} {description} (not found){Colors.RESET}")
    
    # Archivos importantes
    print(f"\n{Colors.BOLD}Important Files:{Colors.RESET}")
    important_files = [
        "netboozt.spec",
        "run_modern.py",
        "requirements.txt",
        "README.md",
        "build.ps1",
    ]
    
    for file_name in important_files:
        path = Path(file_name)
        if path.exists():
            size_kb = path.stat().st_size / 1024
            print(f"{Colors.SUCCESS}✓{Colors.RESET} {file_name:<20} {Colors.DIM}({size_kb:.1f} KB){Colors.RESET}")
        else:
            print(f"{Colors.DIM}✗ {file_name:<20} (not found){Colors.RESET}")
    
    # Mostrar logs si existen
    logs_dir = Path("logs")
    if logs_dir.exists():
        log_files = sorted(logs_dir.glob("netboozt_*.log"), reverse=True)
        if log_files:
            print(f"\n{Colors.BOLD}Recent Logs:{Colors.RESET}")
            for log_file in log_files[:5]:
                size_kb = log_file.stat().st_size / 1024
                modified = log_file.stat().st_mtime
                import datetime
                mod_time = datetime.datetime.fromtimestamp(modified).strftime("%Y-%m-%d %H:%M:%S")
                print(f"  {Colors.INFO}•{Colors.RESET} {log_file.name:<35} {Colors.DIM}{size_kb:>6.1f} KB  {mod_time}{Colors.RESET}")

def action_report():
    """Reportar bug o feedback vía email"""
    import webbrowser
    import urllib.parse
    
    print(f"\n{Colors.INFO}▶{Colors.RESET} Abriendo cliente de email...")
    
    subject = "[NetBoozt] Bug Report / Feedback"
    body = """Hola equipo LOUST,

[ ] Reportar Bug
[ ] Sugerencia de Feature
[ ] Colaborar con código
[ ] Feedback general

**Descripción**:


**Pasos para reproducir** (si es bug):
1. 
2. 
3. 

**Sistema**:
- Windows version: 
- NetBoozt version: 2.1

**Logs**: [Por favor adjunta el archivo de log si es relevante]

**Comentarios adicionales**:


---
Gracias por NetBoozt. Estoy interesado en colaborar con el proyecto.

Saludos,
"""
    
    mailto_url = f"mailto:opensource@loust.pro?subject={urllib.parse.quote(subject)}&body={urllib.parse.quote(body)}"
    
    try:
        webbrowser.open(mailto_url)
        print(f"{Colors.SUCCESS}✓{Colors.RESET} Email client opened")
        print(f"{Colors.DIM}  To: opensource@loust.pro{Colors.RESET}")
        print(f"\n{Colors.PRIMARY}💡 Tip:{Colors.RESET} Adjunta el archivo de log (opción 'l' en el menú)")
    except Exception as e:
        print(f"{Colors.ERROR}✗{Colors.RESET} Error: {e}")

def action_diagnose():
    """Ejecutar diagnóstico completo de red"""
    print(f"\n{Colors.BOLD}🔍 Running Full Network Diagnostic...{Colors.RESET}\n")
    
    try:
        # Importar módulo de diagnóstico
        sys.path.insert(0, str(Path(__file__).parent))
        from src.monitoring.network_diagnostics import NetworkDiagnostics
        
        diag = NetworkDiagnostics()
        print(diag.get_diagnostic_report())
        
    except ImportError as e:
        print(f"{Colors.ERROR}✗{Colors.RESET} Error importing diagnostics: {e}")
        print(f"{Colors.DIM}Running basic diagnostic...{Colors.RESET}\n")
        
        # Fallback a diagnóstico básico
        tests = [
            ("Adapter Status", "Get-NetAdapter | Where-Object Status -eq 'Up' | Select-Object Name, Status, LinkSpeed"),
            ("Gateway Ping", "ping -n 2 -w 1000 $(Get-NetRoute -DestinationPrefix 0.0.0.0/0 | Select-Object -First 1 -ExpandProperty NextHop)"),
            ("Internet Check", "ping -n 3 8.8.8.8"),
            ("DNS Resolution", "nslookup google.com 8.8.8.8"),
        ]
        
        for name, cmd in tests:
            print(f"{Colors.INFO}▶{Colors.RESET} {name}...")
            result = subprocess.run(f"powershell -Command \"{cmd}\"", shell=True, capture_output=True, text=True)
            if result.returncode == 0:
                print(f"{Colors.SUCCESS}✓{Colors.RESET} {name}: OK")
            else:
                print(f"{Colors.ERROR}✗{Colors.RESET} {name}: FAILED")

def action_dns_test():
    """Benchmark de servidores DNS"""
    print(f"\n{Colors.BOLD}🚀 DNS Server Benchmark{Colors.RESET}\n")
    
    dns_servers = {
        'Cloudflare': '1.1.1.1',
        'Google': '8.8.8.8',
        'Quad9': '9.9.9.9',
        'OpenDNS': '208.67.222.222',
        'AdGuard': '94.140.14.14',
    }
    
    results = []
    
    for name, ip in dns_servers.items():
        print(f"{Colors.DIM}Testing {name} ({ip})...{Colors.RESET}", end='', flush=True)
        
        # Ping test
        ping_cmd = f"ping -n 3 -w 1000 {ip}"
        ping_result = subprocess.run(ping_cmd, shell=True, capture_output=True, text=True)
        
        ping_ms = None
        if ping_result.returncode == 0:
            import re
            match = re.search(r'(?:Media|Average)\s*=\s*(\d+)ms', ping_result.stdout, re.IGNORECASE)
            if match:
                ping_ms = int(match.group(1))
        
        # DNS resolution test
        dns_cmd = f"powershell -Command \"Measure-Command {{ nslookup google.com {ip} 2>$null }} | Select-Object -ExpandProperty TotalMilliseconds\""
        dns_result = subprocess.run(dns_cmd, shell=True, capture_output=True, text=True)
        
        dns_ms = None
        try:
            dns_ms = int(float(dns_result.stdout.strip()))
        except:
            pass
        
        status = "✓" if ping_ms and dns_ms and ping_ms < 100 else "⚠" if ping_ms else "✗"
        results.append((name, ip, ping_ms, dns_ms, status))
        print(f" {status}")
    
    # Mostrar tabla de resultados
    print(f"\n{Colors.BOLD}Results:{Colors.RESET}\n")
    print(f"{'Provider':<12} {'IP':<18} {'Ping (ms)':<12} {'Resolve (ms)':<14} {'Status'}")
    print("-" * 65)
    
    # Ordenar por ping
    results.sort(key=lambda x: (x[2] if x[2] else 9999))
    
    for name, ip, ping_ms, dns_ms, status in results:
        ping_str = f"{ping_ms}" if ping_ms else "TIMEOUT"
        dns_str = f"{dns_ms}" if dns_ms else "FAILED"
        
        if status == "✓":
            color = Colors.SUCCESS
        elif status == "⚠":
            color = Colors.WARNING
        else:
            color = Colors.ERROR
        
        print(f"{color}{name:<12}{Colors.RESET} {ip:<18} {ping_str:<12} {dns_str:<14} {color}{status}{Colors.RESET}")
    
    # Recomendar el mejor
    best = results[0]
    print(f"\n{Colors.SUCCESS}💡 Recommended:{Colors.RESET} {best[0]} ({best[1]}) - {best[2]}ms ping")

def action_win_events():
    """Mostrar eventos de red de Windows"""
    print(f"\n{Colors.BOLD}📋 Windows Network Events (Last Hour){Colors.RESET}\n")
    
    cmd = """
    Get-WinEvent -FilterHashtable @{LogName='System'; ProviderName='Microsoft-Windows-DNS-Client','Microsoft-Windows-WLAN-AutoConfig'; Level=2,3} -MaxEvents 20 2>$null | 
    Select-Object TimeCreated, ProviderName, LevelDisplayName, @{N='Message';E={$_.Message.Substring(0,[Math]::Min(80,$_.Message.Length))}} |
    Format-Table -AutoSize
    """
    
    result = subprocess.run(f"powershell -Command \"{cmd}\"", shell=True, capture_output=True, text=True)
    
    if result.stdout.strip():
        print(result.stdout)
        
        # Contar eventos DNS
        dns_count_cmd = """
        (Get-WinEvent -FilterHashtable @{LogName='System'; ProviderName='Microsoft-Windows-DNS-Client'} -MaxEvents 100 2>$null | 
        Where-Object {$_.TimeCreated -gt (Get-Date).AddHours(-1)}).Count
        """
        dns_result = subprocess.run(f"powershell -Command \"{dns_count_cmd}\"", shell=True, capture_output=True, text=True)
        
        try:
            dns_errors = int(dns_result.stdout.strip())
            if dns_errors > 10:
                print(f"\n{Colors.ERROR}⚠ WARNING: {dns_errors} DNS issues in last hour!{Colors.RESET}")
                print(f"{Colors.DIM}Consider changing DNS servers with option 'f'{Colors.RESET}")
            elif dns_errors > 0:
                print(f"\n{Colors.WARNING}ℹ {dns_errors} DNS events in last hour{Colors.RESET}")
            else:
                print(f"\n{Colors.SUCCESS}✓ No DNS issues detected{Colors.RESET}")
        except:
            pass
    else:
        print(f"{Colors.SUCCESS}✓ No recent network issues found{Colors.RESET}")

def action_fix_dns():
    """Aplicar configuración DNS óptima"""
    print(f"\n{Colors.BOLD}🔧 DNS Optimization{Colors.RESET}\n")
    
    if not is_admin():
        print(f"{Colors.WARNING}⚠ This action requires Administrator privileges{Colors.RESET}")
        print(f"{Colors.DIM}A UAC prompt will appear if you continue{Colors.RESET}\n")
    
    print(f"{Colors.INFO}Current DNS configuration:{Colors.RESET}")
    subprocess.run("powershell -Command \"Get-DnsClientServerAddress -AddressFamily IPv4 | Where-Object ServerAddresses | Format-Table InterfaceAlias, ServerAddresses -AutoSize\"", shell=True)
    
    print(f"\n{Colors.BOLD}Select DNS provider:{Colors.RESET}\n")
    print(f"  {Colors.PRIMARY}1{Colors.RESET} › Cloudflare (1.1.1.1) - Fastest")
    print(f"  {Colors.PRIMARY}2{Colors.RESET} › Google (8.8.8.8) - Most reliable")
    print(f"  {Colors.PRIMARY}3{Colors.RESET} › Quad9 (9.9.9.9) - Security focused")
    print(f"  {Colors.PRIMARY}4{Colors.RESET} › OpenDNS (208.67.222.222)")
    print(f"  {Colors.PRIMARY}5{Colors.RESET} › Reset to DHCP (automatic)")
    print(f"  {Colors.PRIMARY}c{Colors.RESET} › Cancel\n")
    
    choice = input(f"{Colors.PRIMARY}>{Colors.RESET} ").strip().lower()
    
    dns_options = {
        '1': ('1.1.1.1', '1.0.0.1', 'Cloudflare'),
        '2': ('8.8.8.8', '8.8.4.4', 'Google'),
        '3': ('9.9.9.9', '149.112.112.112', 'Quad9'),
        '4': ('208.67.222.222', '208.67.220.220', 'OpenDNS'),
    }
    
    if choice == 'c':
        print(f"{Colors.DIM}Cancelled{Colors.RESET}")
        return
    
    # Obtener adaptador activo
    adapter_cmd = "Get-NetAdapter | Where-Object Status -eq 'Up' | Select-Object -First 1 -ExpandProperty Name"
    adapter_result = subprocess.run(f"powershell -Command \"{adapter_cmd}\"", shell=True, capture_output=True, text=True)
    adapter = adapter_result.stdout.strip()
    
    if not adapter:
        print(f"{Colors.ERROR}✗ No active network adapter found{Colors.RESET}")
        return
    
    print(f"\n{Colors.INFO}Applying to adapter: {adapter}{Colors.RESET}")
    
    if choice == '5':
        # Reset a DHCP
        cmd = f"powershell -Command \"Start-Process powershell -ArgumentList '-Command', 'Set-DnsClientServerAddress -InterfaceAlias \\\"{adapter}\\\" -ResetServerAddresses; Write-Host Done' -Verb RunAs -Wait\""
        subprocess.run(cmd, shell=True)
        print(f"{Colors.SUCCESS}✓ DNS reset to DHCP{Colors.RESET}")
    elif choice in dns_options:
        primary, secondary, name = dns_options[choice]
        cmd = f"powershell -Command \"Start-Process powershell -ArgumentList '-Command', 'Set-DnsClientServerAddress -InterfaceAlias \\\"{adapter}\\\" -ServerAddresses {primary},{secondary}; Write-Host Done' -Verb RunAs -Wait\""
        subprocess.run(cmd, shell=True)
        print(f"{Colors.SUCCESS}✓ DNS changed to {name} ({primary}){Colors.RESET}")
        
        # Flush DNS cache
        print(f"{Colors.INFO}▶ Flushing DNS cache...{Colors.RESET}")
        subprocess.run("ipconfig /flushdns", shell=True, capture_output=True)
        print(f"{Colors.SUCCESS}✓ DNS cache cleared{Colors.RESET}")
    else:
        print(f"{Colors.WARNING}Invalid option{Colors.RESET}")

def main():
    """Main CLI loop"""
    # Cambiar al directorio correcto
    script_dir = Path(__file__).parent
    os.chdir(script_dir)
    
    # Mostrar pantalla de bienvenida (solo primera vez)
    if '--no-welcome' not in sys.argv:
        print_welcome()
    
    # Banner con indicador de admin
    print_banner()
    
    while True:
        print_menu()
        
        try:
            choice = input(f"{Colors.PRIMARY}>{Colors.RESET} ").strip().lower()
            
            if choice == 'q' or choice == 'quit':
                print(f"\n{Colors.DIM}Goodbye! 👋{Colors.RESET}\n")
                break
            
            elif choice == '1' or choice == 'build':
                action_build()
            
            elif choice == '2' or choice == 'dev':
                action_dev()
            
            elif choice == '3' or choice == 'rebuild':
                action_rebuild()
            
            elif choice == '4' or choice == 'run':
                action_run()
            
            elif choice == '5' or choice == 'deploy':
                action_deploy()
            
            elif choice == '6' or choice == 'github':
                action_github()
            
            elif choice == '7' or choice == 'web':
                action_web()
            
            elif choice == 'r' or choice == 'report':
                action_report()
            
            elif choice == 'l' or choice == 'logs':
                action_logs()
            
            elif choice == 't' or choice == 'tree':
                action_tree()
            
            elif choice == 'c' or choice == 'clean':
                action_clean()
            
            elif choice == 'v' or choice == 'verify':
                action_verify()
            
            elif choice == 'd' or choice == 'diagnose':
                action_diagnose()
            
            elif choice == 'n' or choice == 'dns':
                action_dns_test()
            
            elif choice == 'w' or choice == 'events':
                action_win_events()
            
            elif choice == 'f' or choice == 'fix':
                action_fix_dns()
            
            else:
                print(f"{Colors.WARNING}⚠{Colors.RESET} Invalid option. Try again.")
            
            # Pausa antes de volver al menú
            input(f"\n{Colors.DIM}Press Enter to continue...{Colors.RESET}")
            print("\n" * 2)
        
        except KeyboardInterrupt:
            print(f"\n\n{Colors.DIM}Goodbye! 👋{Colors.RESET}\n")
            break
        except EOFError:
            print(f"\n\n{Colors.DIM}Goodbye! 👋{Colors.RESET}\n")
            break

if __name__ == "__main__":
    main()

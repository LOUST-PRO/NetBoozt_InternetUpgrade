"""
NetBoozt - Error Dialog
Diálogo de error con opciones de soporte y reporte

By LOUST (www.loust.pro)
"""

import customtkinter as ctk
import webbrowser
import urllib.parse
from pathlib import Path
import subprocess

try:
    from .theme import *
except ImportError:
    BG_MAIN = "#0a0a0a"
    PRIMARY = "#00d4aa"
    TEXT_PRIMARY = "#ffffff"
    ERROR_BG = "#2a0a0a"


class ErrorDialog(ctk.CTkToplevel):
    """Diálogo de error con acciones de soporte"""
    
    def __init__(self, parent, error_title: str, error_message: str, error_details: str = None):
        super().__init__(parent)
        
        self.error_details = error_details
        
        # Window configuration
        self.title("NetBoozt - Error")
        self.geometry("600x450")
        self.resizable(False, False)
        
        # Center on screen
        self.update_idletasks()
        x = (self.winfo_screenwidth() // 2) - (600 // 2)
        y = (self.winfo_screenheight() // 2) - (450 // 2)
        self.geometry(f"+{x}+{y}")
        
        # Configure colors
        self.configure(fg_color=BG_MAIN)
        
        # Keep on top
        self.attributes('-topmost', True)
        self.lift()
        self.focus_force()
        
        # Main container
        main_frame = ctk.CTkFrame(
            self,
            fg_color=BG_MAIN,
            corner_radius=0
        )
        main_frame.pack(fill="both", expand=True, padx=20, pady=20)
        
        # Error icon and title
        header_frame = ctk.CTkFrame(main_frame, fg_color=ERROR_BG, corner_radius=10)
        header_frame.pack(fill="x", pady=(0, 20))
        
        error_icon = ctk.CTkLabel(
            header_frame,
            text="⚠️",
            font=ctk.CTkFont(size=48)
        )
        error_icon.pack(pady=(15, 5))
        
        title_label = ctk.CTkLabel(
            header_frame,
            text=error_title,
            font=ctk.CTkFont(size=20, weight="bold"),
            text_color="#ff4444"
        )
        title_label.pack(pady=(0, 15))
        
        # Error message
        message_frame = ctk.CTkFrame(main_frame, fg_color="#1a1a1a", corner_radius=10)
        message_frame.pack(fill="both", expand=True, pady=(0, 20))
        
        message_label = ctk.CTkLabel(
            message_frame,
            text=error_message,
            font=ctk.CTkFont(size=13),
            text_color=TEXT_PRIMARY,
            wraplength=520,
            justify="left"
        )
        message_label.pack(pady=20, padx=20)
        
        if error_details:
            details_label = ctk.CTkLabel(
                message_frame,
                text=f"Details: {error_details[:200]}...",
                font=ctk.CTkFont(size=10),
                text_color="#888888",
                wraplength=520,
                justify="left"
            )
            details_label.pack(pady=(0, 20), padx=20)
        
        # Action buttons
        buttons_frame = ctk.CTkFrame(main_frame, fg_color="transparent")
        buttons_frame.pack(fill="x")
        
        # Ver Logs
        logs_btn = ctk.CTkButton(
            buttons_frame,
            text="📄 Ver Logs",
            command=self.open_logs,
            fg_color="#2a2a2a",
            hover_color="#3a3a3a",
            width=120,
            height=36
        )
        logs_btn.pack(side="left", padx=5)
        
        # Reportar Bug
        report_btn = ctk.CTkButton(
            buttons_frame,
            text="📧 Reportar Bug",
            command=self.report_bug,
            fg_color="#ff6600",
            hover_color="#ff8833",
            width=140,
            height=36
        )
        report_btn.pack(side="left", padx=5)
        
        # GitHub
        github_btn = ctk.CTkButton(
            buttons_frame,
            text="🔧 GitHub",
            command=self.open_github,
            fg_color=PRIMARY,
            hover_color="#00ffbb",
            width=120,
            height=36
        )
        github_btn.pack(side="left", padx=5)
        
        # Cerrar
        close_btn = ctk.CTkButton(
            buttons_frame,
            text="Cerrar",
            command=self.close_dialog,
            fg_color="#444444",
            hover_color="#555555",
            width=100,
            height=36
        )
        close_btn.pack(side="right", padx=5)
        
        # Footer
        footer = ctk.CTkLabel(
            main_frame,
            text="Agradecemos tu colaboración reportando errores • www.loust.pro",
            font=ctk.CTkFont(size=9),
            text_color="#404040"
        )
        footer.pack(side="bottom", pady=(10, 0))
    
    def open_logs(self):
        """Abrir archivo de logs más reciente"""
        try:
            # FIX: Usar ruta de logger para obtener directorio correcto
            from utils.logger import LOGS_DIR
            
            if LOGS_DIR.exists():
                log_files = sorted(LOGS_DIR.glob("netboozt_*.log"), reverse=True)
                if log_files:
                    subprocess.Popen(['notepad.exe', str(log_files[0].absolute())])
                else:
                    self.show_info("No hay archivos de log disponibles")
            else:
                self.show_info(f"Directorio de logs no encontrado: {LOGS_DIR}")
        except Exception as e:
            self.show_info(f"Error abriendo logs: {e}")
    
    def report_bug(self):
        """Abrir mailto con plantilla de reporte"""
        subject = "[NetBoozt Bug Report] Error en inicialización"
        
        body = f"""Hola equipo LOUST,

Encontré un error al ejecutar NetBoozt:

**Error**: {self.error_details if self.error_details else 'Ver adjunto'}

**Pasos para reproducir**:
1. 
2. 
3. 

**Sistema**:
- Windows version: 
- NetBoozt version: 2.1

**Logs adjuntos**: [Por favor adjunta el archivo de log]

**Comentarios adicionales**:


---
Gracias por tu trabajo en NetBoozt. Estoy interesado en:
[ ] Colaborar con código
[ ] Reportar más bugs
[ ] Sugerencias de features
[ ] Feedback general

Saludos,
"""
        
        mailto_url = f"mailto:opensource@loust.pro?subject={urllib.parse.quote(subject)}&body={urllib.parse.quote(body)}"
        webbrowser.open(mailto_url)
    
    def open_github(self):
        """Abrir repositorio en GitHub"""
        webbrowser.open("https://github.com/LOUST-PRO/NetBoozt_InternetUpgrade")
    
    def close_dialog(self):
        """Cerrar diálogo"""
        self.destroy()
    
    def show_info(self, message: str):
        """Mostrar mensaje informativo"""
        info_window = ctk.CTkToplevel(self)
        info_window.title("Info")
        info_window.geometry("300x100")
        
        label = ctk.CTkLabel(info_window, text=message, wraplength=250)
        label.pack(pady=20)
        
        btn = ctk.CTkButton(info_window, text="OK", command=info_window.destroy)
        btn.pack(pady=10)


def show_error_dialog(parent, title: str, message: str, details: str = None):
    """
    Mostrar diálogo de error modal
    
    Args:
        parent: Ventana padre
        title: Título del error
        message: Mensaje de error
        details: Detalles técnicos del error
    """
    dialog = ErrorDialog(parent, title, message, details)
    dialog.grab_set()  # Modal
    dialog.wait_window()  # Block until closed


# Test
if __name__ == "__main__":
    root = ctk.CTk()
    root.withdraw()
    
    show_error_dialog(
        root,
        "Error al cargar módulos",
        "No se pudo inicializar la aplicación correctamente.\n\nPor favor revisa los logs para más información.",
        "ImportError: No module named 'PIL'"
    )
    
    root.destroy()

"""
NetBoozt - Splash Screen con Loading Animation
Pantalla de carga que se muestra mientras se inicializa la aplicación

By LOUST (www.loust.pro)
"""

import customtkinter as ctk
from pathlib import Path
import threading
import time

try:
    from .theme import *
except ImportError:
    # Fallback colors if theme not available
    BG_MAIN = "#0a0a0a"
    PRIMARY = "#00d4aa"
    TEXT_PRIMARY = "#ffffff"
    TEXT_SECONDARY = "#a0a0a0"


class SplashScreen(ctk.CTkToplevel):
    """Splash screen with animated loader"""
    
    def __init__(self, parent=None):
        super().__init__(parent)
        
        # Window configuration
        self.title("NetBoozt")
        self.geometry("500x400")
        self.resizable(False, False)
        
        # Remove window decorations
        self.overrideredirect(True)
        
        # Center on screen
        self.update_idletasks()
        x = (self.winfo_screenwidth() // 2) - (500 // 2)
        y = (self.winfo_screenheight() // 2) - (400 // 2)
        self.geometry(f"+{x}+{y}")
        
        # Configure colors
        self.configure(fg_color=BG_MAIN)
        
        # Main container
        self.main_frame = ctk.CTkFrame(
            self,
            fg_color=BG_MAIN,
            corner_radius=15,
            border_width=2,
            border_color=PRIMARY
        )
        self.main_frame.pack(fill="both", expand=True, padx=2, pady=2)
        
        # Logo LOUST estático (sin rotación, sin distorsión)
        try:
            from pathlib import Path
            from PIL import Image, ImageTk
            
            # Cargar logo LOUST
            logo_path = Path(__file__).parent.parent.parent / "assets" / "loust_logo.png"
            if logo_path.exists():
                original_logo = Image.open(logo_path)
                # Redimensionar manteniendo aspect ratio
                original_logo = original_logo.resize((100, 100), Image.Resampling.LANCZOS)
                self.logo_photo = ImageTk.PhotoImage(original_logo)
                
                # Logo estático (sin variables de animación)
                self.logo_label = ctk.CTkLabel(
                    self.main_frame,
                    text="",
                    image=self.logo_photo
                )
                self.logo_label.pack(pady=(50, 20))
                self.has_logo = True
            else:
                # Fallback: emoji
                self.logo_label = ctk.CTkLabel(
                    self.main_frame,
                    text="🚀",
                    font=ctk.CTkFont(size=80)
                )
                self.logo_label.pack(pady=(50, 20))
                self.has_logo = False
        except Exception as e:
            import sys
            sys.path.insert(0, str(Path(__file__).parent.parent))
            from utils.logger import log_error
            log_error(f"Splash logo error: {e}")
            # Fallback en caso de error
            self.logo_label = ctk.CTkLabel(
                self.main_frame,
                text="🚀",
                font=ctk.CTkFont(size=80)
            )
            self.logo_label.pack(pady=(50, 20))
            self.has_logo = False
        
        # Title
        self.title_label = ctk.CTkLabel(
            self.main_frame,
            text="NetBoozt",
            font=ctk.CTkFont(size=32, weight="bold"),
            text_color=PRIMARY
        )
        self.title_label.pack(pady=(0, 5))
        
        # Subtitle
        self.subtitle_label = ctk.CTkLabel(
            self.main_frame,
            text="Network Optimizer & Failover",
            font=ctk.CTkFont(size=14),
            text_color=TEXT_SECONDARY
        )
        self.subtitle_label.pack(pady=(0, 40))
        
        # Loading progress bar
        self.progress_bar = ctk.CTkProgressBar(
            self.main_frame,
            width=300,
            height=6,
            corner_radius=3,
            progress_color=PRIMARY,
            fg_color="#1a1a1a"
        )
        self.progress_bar.pack(pady=(0, 15))
        self.progress_bar.set(0)
        
        # Status label
        self.status_label = ctk.CTkLabel(
            self.main_frame,
            text="Inicializando...",
            font=ctk.CTkFont(size=12),
            text_color=TEXT_SECONDARY
        )
        self.status_label.pack(pady=(0, 10))
        
        # Version label con link a LOUST
        self.version_frame = ctk.CTkFrame(
            self.main_frame,
            fg_color="transparent"
        )
        self.version_frame.pack(side="bottom", pady=15)
        
        # By LOUST (clicable)
        self.version_label = ctk.CTkLabel(
            self.version_frame,
            text="v2.1 • By LOUST",
            font=ctk.CTkFont(size=10),
            text_color="#00d4aa",
            cursor="hand2"
        )
        self.version_label.pack(side="left", padx=5)
        self.version_label.bind("<Button-1>", lambda e: self.open_url("https://www.loust.pro"))
        
        # GitHub icon (clicable)
        self.github_label = ctk.CTkLabel(
            self.version_frame,
            text="⚙",  # GitHub icon
            font=ctk.CTkFont(size=14),
            text_color="#404040",
            cursor="hand2"
        )
        self.github_label.pack(side="left", padx=5)
        self.github_label.bind("<Button-1>", lambda e: self.open_url("https://github.com/LOUST-PRO/NetBoozt_InternetUpgrade"))
        
        # Botón de cerrar (esquina superior derecha)
        self.close_button = ctk.CTkButton(
            self.main_frame,
            text="✕",
            width=30,
            height=30,
            corner_radius=15,
            fg_color="transparent",
            hover_color="#2a0a0a",
            text_color="#808080",
            font=ctk.CTkFont(size=16, weight="bold"),
            command=self.force_close
        )
        self.close_button.place(x=460, y=10)
        
        # Animation state
        self.animation_running = False
        self.progress_value = 0
        self._after_ids = []  # Track after callbacks for cleanup
        
        # Keep window on top
        self.attributes('-topmost', True)
        
        # Start loading animation
        self.start_animation()
    
    def open_url(self, url):
        """Abrir URL en navegador"""
        import webbrowser
        webbrowser.open(url)
    
    def force_close(self):
        """Cerrar splash forzadamente"""
        self.animation_running = False
        # Cancel all pending after callbacks
        for after_id in self._after_ids:
            try:
                self.after_cancel(after_id)
            except Exception:
                pass
        self._after_ids.clear()
        self.destroy()
    
    def start_animation(self):
        """Start progress bar animation"""
        self.animation_running = True
        self.animate_progress()
        # NO hay animación de logo - es estático
    
    def animate_progress(self):
        """Animate the progress bar"""
        if not self.animation_running:
            return
        
        # Smooth progress animation
        if self.progress_value < 0.9:
            self.progress_value += 0.02
            self.progress_bar.set(self.progress_value)
        
        # Update after 50ms and track the ID
        after_id = self.after(50, self.animate_progress)
        self._after_ids.append(after_id)
    
    def update_status(self, status_text: str, progress: float = None):
        """Update status text and optionally progress"""
        self.status_label.configure(text=status_text)
        if progress is not None:
            self.progress_value = min(progress, 1.0)
            self.progress_bar.set(self.progress_value)
    
    def finish(self):
        """Complete loading and close splash"""
        # Animate to 100%
        self.progress_value = 1.0
        self.progress_bar.set(1.0)
        self.status_label.configure(text="✓ Listo")
        
        # Wait a bit then close and track the ID
        after_id = self.after(300, self.close_splash)
        self._after_ids.append(after_id)
    
    def close_splash(self):
        """Close the splash screen"""
        self.animation_running = False
        # Cancel all pending after callbacks
        for after_id in self._after_ids:
            try:
                self.after_cancel(after_id)
            except Exception:
                pass
        self._after_ids.clear()
        self.destroy()


def show_splash_while_loading(load_function, *args, **kwargs):
    """
    Show splash screen while executing a loading function
    
    Args:
        load_function: Function to execute while splash is shown
        *args, **kwargs: Arguments for load_function
    
    Returns:
        Result of load_function
    """
    # Create root window (hidden)
    root = ctk.CTk()
    root.withdraw()
    
    # Create splash
    splash = SplashScreen(root)
    
    result = None
    error = None
    
    def loading_thread():
        nonlocal result, error
        try:
            # Execute loading function
            result = load_function(*args, **kwargs)
        except Exception as e:
            error = e
        finally:
            # Close splash when done
            splash.after(100, splash.finish)
    
    # Start loading in background thread
    thread = threading.Thread(target=loading_thread, daemon=True)
    thread.start()
    
    # Show splash (blocks until closed)
    splash.mainloop()
    
    # Wait for thread to complete
    thread.join(timeout=2)
    
    # Return result or raise error
    if error:
        raise error
    return result


# Example usage
if __name__ == "__main__":
    import time
    
    def mock_loading():
        """Simulate loading tasks"""
        time.sleep(0.5)
        # Simulate initialization tasks
        time.sleep(0.5)
        return "App loaded!"
    
    result = show_splash_while_loading(mock_loading)
    # Result handled by caller

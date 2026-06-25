"""
NetBoozt - GitHub Tab
Información del repositorio, contribución y collaboration

By LOUST (www.loust.pro)
"""

import customtkinter as ctk
import webbrowser
from .theme import *


class GitHubTab(ctk.CTkScrollableFrame):
    """Tab con información de GitHub y colaboración"""
    
    def __init__(self, parent):
        super().__init__(
            parent,
            fg_color=BG_MAIN,
            scrollbar_button_color=SCROLLBAR_COLOR,
            scrollbar_button_hover_color=SCROLLBAR_HOVER
        )
        
        self.repo_url = "https://github.com/LOUST-PRO/NetBoozt_InternetUpgrade"
        self.author_url = "https://github.com/louzt"
        self.website_url = "https://loust.pro"
        
        self._create_ui()
    
    def _create_ui(self):
        """Crear interfaz"""
        # Hero section
        hero = ctk.CTkFrame(self, fg_color=BG_SIDEBAR, corner_radius=RADIUS_LARGE, height=200)
        hero.pack(fill="x", padx=PADDING_LARGE, pady=PADDING_LARGE)
        
        ctk.CTkLabel(
            hero,
            text="🔗",
            font=ctk.CTkFont(size=80)
        ).pack(pady=(30, 10))
        
        ctk.CTkLabel(
            hero,
            text="NetBoozt on GitHub",
            font=ctk.CTkFont(size=FONT_SIZE_HERO, weight="bold"),
            text_color=PRIMARY
        ).pack()
        
        ctk.CTkLabel(
            hero,
            text="Open Source Network Optimizer",
            font=ctk.CTkFont(size=FONT_SIZE_NORMAL),
            text_color=TEXT_SECONDARY
        ).pack(pady=5)
        
        # Links principales
        links_frame = ctk.CTkFrame(self, fg_color="transparent")
        links_frame.pack(fill="x", padx=PADDING_LARGE, pady=PADDING_NORMAL)
        
        self._add_link_button(
            links_frame,
            "🌐 Ver Repositorio",
            self.repo_url,
            PRIMARY
        ).pack(side="left", expand=True, padx=5)
        
        self._add_link_button(
            links_frame,
            "⭐ Star en GitHub",
            self.repo_url + "/stargazers",
            "#ffaa00"
        ).pack(side="left", expand=True, padx=5)
        
        self._add_link_button(
            links_frame,
            "🐛 Reportar Bug",
            self.repo_url + "/issues/new",
            "#ff4444"
        ).pack(side="left", expand=True, padx=5)
        
        # Stats del repo (simuladas)
        stats_container = ctk.CTkFrame(self, fg_color="transparent")
        stats_container.pack(fill="x", padx=PADDING_LARGE, pady=PADDING_LARGE)
        
        stats = [
            ("⭐", "Stars", "24", "En GitHub"),
            ("🔀", "Forks", "7", "Contribuciones"),
            ("👁️", "Watchers", "12", "Siguiendo"),
            ("📝", "Issues", "3", "Abiertas"),
        ]
        
        for icon, label, value, sublabel in stats:
            self._add_stat_card(stats_container, icon, label, value, sublabel).pack(
                side="left", expand=True, padx=5
            )
        
        # Sección: Cómo contribuir
        contrib_section = ctk.CTkFrame(self, fg_color=BG_CARD, corner_radius=RADIUS_LARGE)
        contrib_section.pack(fill="x", padx=PADDING_LARGE, pady=PADDING_NORMAL)
        
        ctk.CTkLabel(
            contrib_section,
            text="🤝 Cómo Contribuir",
            font=ctk.CTkFont(size=FONT_SIZE_HEADER, weight="bold"),
            text_color=PRIMARY,
            anchor="w"
        ).pack(fill="x", padx=PADDING_LARGE, pady=(PADDING_LARGE, PADDING_SMALL))
        
        steps = [
            ("1", "Fork el repositorio", "Crea tu copia personal"),
            ("2", "Crea una branch", "git checkout -b feature/amazing-feature"),
            ("3", "Commit tus cambios", "git commit -m 'Add amazing feature'"),
            ("4", "Push a GitHub", "git push origin feature/amazing-feature"),
            ("5", "Abre un Pull Request", "¡Describe tus cambios!"),
        ]
        
        for num, title, desc in steps:
            self._add_step(contrib_section, num, title, desc)
        
        # Sección: Tecnologías
        tech_section = ctk.CTkFrame(self, fg_color=BG_CARD, corner_radius=RADIUS_LARGE)
        tech_section.pack(fill="x", padx=PADDING_LARGE, pady=PADDING_NORMAL)
        
        ctk.CTkLabel(
            tech_section,
            text="🛠️ Stack Tecnológico",
            font=ctk.CTkFont(size=FONT_SIZE_HEADER, weight="bold"),
            text_color=PRIMARY,
            anchor="w"
        ).pack(fill="x", padx=PADDING_LARGE, pady=(PADDING_LARGE, PADDING_SMALL))
        
        tech_grid = ctk.CTkFrame(tech_section, fg_color="transparent")
        tech_grid.pack(fill="x", padx=PADDING_LARGE, pady=PADDING_LARGE)
        
        technologies = [
            ("🐍", "Python 3.10+", "Core Language"),
            ("🎨", "CustomTkinter", "Modern GUI"),
            ("📊", "Matplotlib", "Real-time Graphs"),
            ("💾", "TinyDB", "Local Storage"),
            ("⚡", "psutil", "System Monitoring"),
            ("📦", "PyInstaller", ".exe Packaging"),
        ]
        
        for i, (icon, name, desc) in enumerate(technologies):
            row = i // 3
            col = i % 3
            self._add_tech_badge(tech_grid, icon, name, desc).grid(
                row=row, column=col, padx=10, pady=10, sticky="nsew"
            )
        
        for i in range(3):
            tech_grid.grid_columnconfigure(i, weight=1)
        
        # Sección: Author
        author_section = ctk.CTkFrame(self, fg_color=BG_SIDEBAR, corner_radius=RADIUS_LARGE)
        author_section.pack(fill="x", padx=PADDING_LARGE, pady=PADDING_LARGE)
        
        ctk.CTkLabel(
            author_section,
            text="👨‍💻",
            font=ctk.CTkFont(size=60)
        ).pack(pady=(20, 10))
        
        ctk.CTkLabel(
            author_section,
            text="Made by LOUST",
            font=ctk.CTkFont(size=FONT_SIZE_HEADER, weight="bold"),
            text_color=PRIMARY
        ).pack()
        
        ctk.CTkLabel(
            author_section,
            text="Network Performance Engineer",
            font=ctk.CTkFont(size=FONT_SIZE_NORMAL),
            text_color=TEXT_SECONDARY
        ).pack(pady=5)
        
        author_links = ctk.CTkFrame(author_section, fg_color="transparent")
        author_links.pack(pady=20)
        
        self._add_small_link_button(author_links, "🌐 loust.pro", self.website_url).pack(side="left", padx=5)
        self._add_small_link_button(author_links, "🔗 GitHub", self.author_url).pack(side="left", padx=5)
        
        # License
        ctk.CTkLabel(
            author_section,
            text="📄 Apache-2.0 License © 2024",
            font=ctk.CTkFont(size=FONT_SIZE_SMALL),
            text_color=TEXT_DISABLED
        ).pack(pady=(0, 20))
    
    def _add_link_button(self, parent, text: str, url: str, color: str):
        """Agregar botón con link"""
        btn = ctk.CTkButton(
            parent,
            text=text,
            command=lambda: webbrowser.open(url),
            fg_color=color,
            hover_color=self._darken_color(color),
            height=40,
            corner_radius=RADIUS_BUTTON,
            font=ctk.CTkFont(size=FONT_SIZE_NORMAL, weight="bold")
        )
        return btn
    
    def _add_small_link_button(self, parent, text: str, url: str):
        """Agregar botón pequeño con link"""
        btn = ctk.CTkButton(
            parent,
            text=text,
            command=lambda: webbrowser.open(url),
            fg_color=BG_CARD,
            hover_color=BG_CARD_HOVER,
            border_width=1,
            border_color=BORDER_DEFAULT,
            text_color=PRIMARY,
            height=32,
            corner_radius=RADIUS_BUTTON,
            font=ctk.CTkFont(size=FONT_SIZE_SMALL)
        )
        return btn
    
    def _add_stat_card(self, parent, icon: str, label: str, value: str, sublabel: str):
        """Agregar tarjeta de estadística"""
        card = ctk.CTkFrame(parent, fg_color=BG_CARD, corner_radius=RADIUS_LARGE)
        
        ctk.CTkLabel(
            card,
            text=icon,
            font=ctk.CTkFont(size=40)
        ).pack(pady=(15, 5))
        
        ctk.CTkLabel(
            card,
            text=value,
            font=ctk.CTkFont(size=FONT_SIZE_HEADER, weight="bold"),
            text_color=PRIMARY
        ).pack()
        
        ctk.CTkLabel(
            card,
            text=label,
            font=ctk.CTkFont(size=FONT_SIZE_NORMAL),
            text_color=TEXT_PRIMARY
        ).pack()
        
        ctk.CTkLabel(
            card,
            text=sublabel,
            font=ctk.CTkFont(size=FONT_SIZE_SMALL),
            text_color=TEXT_SECONDARY
        ).pack(pady=(0, 15))
        
        return card
    
    def _add_step(self, parent, num: str, title: str, desc: str):
        """Agregar paso de contribución"""
        step = ctk.CTkFrame(parent, fg_color="transparent")
        step.pack(fill="x", padx=PADDING_LARGE, pady=5)
        
        # Número
        num_label = ctk.CTkLabel(
            step,
            text=num,
            font=ctk.CTkFont(size=FONT_SIZE_LARGE, weight="bold"),
            text_color=PRIMARY,
            width=40,
            fg_color=BG_SIDEBAR,
            corner_radius=20
        )
        num_label.pack(side="left", padx=(0, 15))
        
        # Contenido
        content = ctk.CTkFrame(step, fg_color="transparent")
        content.pack(side="left", fill="x", expand=True)
        
        ctk.CTkLabel(
            content,
            text=title,
            font=ctk.CTkFont(size=FONT_SIZE_NORMAL, weight="bold"),
            text_color=TEXT_PRIMARY,
            anchor="w"
        ).pack(fill="x")
        
        ctk.CTkLabel(
            content,
            text=desc,
            font=ctk.CTkFont(family=FONT_FAMILY_MONO, size=FONT_SIZE_SMALL),
            text_color=TEXT_SECONDARY,
            anchor="w"
        ).pack(fill="x")
    
    def _add_tech_badge(self, parent, icon: str, name: str, desc: str):
        """Agregar badge de tecnología"""
        badge = ctk.CTkFrame(parent, fg_color=BG_SIDEBAR, corner_radius=RADIUS_NORMAL)
        
        ctk.CTkLabel(
            badge,
            text=icon,
            font=ctk.CTkFont(size=30)
        ).pack(pady=(10, 5))
        
        ctk.CTkLabel(
            badge,
            text=name,
            font=ctk.CTkFont(size=FONT_SIZE_NORMAL, weight="bold"),
            text_color=TEXT_PRIMARY
        ).pack()
        
        ctk.CTkLabel(
            badge,
            text=desc,
            font=ctk.CTkFont(size=FONT_SIZE_SMALL),
            text_color=TEXT_SECONDARY
        ).pack(pady=(0, 10))
        
        return badge
    
    @staticmethod
    def _darken_color(hex_color: str) -> str:
        """Oscurecer color hexadecimal"""
        # Simple darkening by reducing RGB values
        hex_color = hex_color.lstrip('#')
        r, g, b = tuple(int(hex_color[i:i+2], 16) for i in (0, 2, 4))
        r, g, b = max(0, r-30), max(0, g-30), max(0, b-30)
        return f'#{r:02x}{g:02x}{b:02x}'

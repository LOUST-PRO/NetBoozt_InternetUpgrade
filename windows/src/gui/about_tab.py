"""
NetBoozt - About/Info Tab
Página minimalista con animaciones y efectos

By LOUST (www.loust.pro)
"""

import customtkinter as ctk
from typing import Optional
import webbrowser
from .theme import (
    BG_MAIN, BG_CARD, BG_HOVER, PRIMARY, TEXT_PRIMARY, TEXT_SECONDARY, TEXT_DISABLED,
    FONT_FAMILY, FONT_SIZE_LARGE, FONT_SIZE_NORMAL, FONT_SIZE_SMALL
)


class AboutTab(ctk.CTkScrollableFrame):
    """Tab About con diseño minimalista y animaciones"""
    
    def __init__(self, master, **kwargs):
        super().__init__(master, fg_color="transparent", **kwargs)
        
        self.animation_running = False
        self.create_content()
    
    def create_content(self):
        """Crear contenido del tab"""
        
        # Hero section con fade-in
        hero = ctk.CTkFrame(self, fg_color=BG_MAIN, corner_radius=12, height=250)
        hero.pack(fill="x", padx=20, pady=(20, 30))
        hero.pack_propagate(False)
        
        # Logo grande (si existe)
        logo_label = ctk.CTkLabel(
            hero,
            text="⚡",
            font=ctk.CTkFont(size=80)
        )
        logo_label.pack(pady=(40, 10))
        
        # Nombre de la app con animación de escritura
        name_label = ctk.CTkLabel(
            hero,
            text="NetBoozt",
            font=ctk.CTkFont(size=42, weight="bold"),
            text_color=PRIMARY
        )
        name_label.pack(pady=(0, 5))
        
        # Tagline
        tagline = ctk.CTkLabel(
            hero,
            text="Network Performance Optimizer",
            font=ctk.CTkFont(size=16),
            text_color=TEXT_SECONDARY
        )
        tagline.pack(pady=(0, 10))
        
        # Version badge
        version_frame = ctk.CTkFrame(hero, fg_color=BG_CARD, corner_radius=6)
        version_frame.pack(pady=10)
        
        version_label = ctk.CTkLabel(
            version_frame,
            text="v2.0.0",
            font=ctk.CTkFont(size=12, weight="bold"),
            text_color=PRIMARY,
            padx=15,
            pady=6
        )
        version_label.pack()
        
        # Stats cards con hover effects
        stats_frame = ctk.CTkFrame(self, fg_color="transparent")
        stats_frame.pack(fill="x", padx=20, pady=20)
        
        stats_frame.grid_columnconfigure((0, 1, 2, 3), weight=1)
        
        stats = [
            ("13+", "Optimizaciones TCP/IP", "#00a8ff"),
            ("7", "DNS Fallback Tiers", "#00ff88"),
            ("100%", "Open Source", "#ffd700"),
            ("24/7", "Monitoreo Real-Time", "#a55eea")
        ]
        
        for idx, (value, label, color) in enumerate(stats):
            card = self._create_stat_card(stats_frame, value, label, color)
            card.grid(row=0, column=idx, padx=10, pady=10, sticky="ew")
        
        # Features section
        features_title = ctk.CTkLabel(
            self,
            text="✨ Características Principales",
            font=ctk.CTkFont(size=20, weight="bold"),
            anchor="w"
        )
        features_title.pack(fill="x", padx=30, pady=(30, 15))
        
        features = [
            ("🚀", "TCP Optimizations", "13+ optimizaciones avanzadas de red"),
            ("📊", "Real-Time Dashboard", "Gráficas en vivo con matplotlib"),
            ("🔄", "DNS Failover", "7 niveles de fallback automático"),
            ("💾", "Local Storage", "TinyDB para historial de tests"),
            ("🎨", "Modern UI", "Dark theme con CustomTkinter"),
            ("🔧", "Auto-Detection", "Detecta optimizaciones activas")
        ]
        
        for icon, title, desc in features:
            self._create_feature_item(icon, title, desc)
        
        # Tech stack
        tech_title = ctk.CTkLabel(
            self,
            text="🛠️ Stack Tecnológico",
            font=ctk.CTkFont(size=20, weight="bold"),
            anchor="w"
        )
        tech_title.pack(fill="x", padx=30, pady=(30, 15))
        
        tech_grid = ctk.CTkFrame(self, fg_color="transparent")
        tech_grid.pack(fill="x", padx=30, pady=10)
        
        tech_grid.grid_columnconfigure((0, 1, 2), weight=1)
        
        techs = [
            ("Python 3.10+", "Language", "#3776ab"),
            ("CustomTkinter 5.2+", "GUI Framework", "#00d4aa"),
            ("Matplotlib 3.8+", "Graphing", "#11557c"),
            ("TinyDB 4.8+", "Database", "#ffd700"),
            ("psutil 7.1+", "Monitoring", "#ff6b6b"),
            ("PyInstaller 6.0+", "Packaging", "#a55eea")
        ]
        
        for idx, (name, category, color) in enumerate(techs):
            row = idx // 3
            col = idx % 3
            tech_card = self._create_tech_card(tech_grid, name, category, color)
            tech_card.grid(row=row, column=col, padx=8, pady=8, sticky="ew")
        
        # Links section
        links_title = ctk.CTkLabel(
            self,
            text="🔗 Enlaces",
            font=ctk.CTkFont(size=20, weight="bold"),
            anchor="w"
        )
        links_title.pack(fill="x", padx=30, pady=(30, 15))
        
        links_frame = ctk.CTkFrame(self, fg_color="transparent")
        links_frame.pack(fill="x", padx=30, pady=10)
        
        # GitHub
        github_btn = self._create_link_button(
            links_frame,
            "🐙 GitHub",
            "github.com/louzt",
            lambda: webbrowser.open("https://github.com/louzt")
        )
        github_btn.pack(side="left", padx=5)
        
        # Portfolio
        portfolio_btn = self._create_link_button(
            links_frame,
            "🌐 Portfolio",
            "loust.pro",
            lambda: webbrowser.open("https://loust.pro")
        )
        portfolio_btn.pack(side="left", padx=5)
        
        # Docs (futuro)
        docs_btn = self._create_link_button(
            links_frame,
            "📚 Docs",
            "Próximamente",
            None,
            disabled=True
        )
        docs_btn.pack(side="left", padx=5)
        
        # Footer
        footer = ctk.CTkFrame(self, fg_color=BG_MAIN, corner_radius=12, height=100)
        footer.pack(fill="x", padx=20, pady=30)
        footer.pack_propagate(False)
        
        footer_text = ctk.CTkLabel(
            footer,
            text="Made with ❤️ by LOUST\n© 2024 - Apache-2.0 License",
            font=ctk.CTkFont(size=12),
            text_color=TEXT_SECONDARY,
            justify="center"
        )
        footer_text.pack(expand=True)
    
    def _create_stat_card(self, parent, value: str, label: str, color: str) -> ctk.CTkFrame:
        """Crear card de estadística con hover effect"""
        card = ctk.CTkFrame(parent, fg_color=BG_CARD, corner_radius=8)
        
        # Bind hover events
        card.bind("<Enter>", lambda e: self._on_card_hover(card, BG_HOVER))
        card.bind("<Leave>", lambda e: self._on_card_hover(card, BG_CARD))
        
        value_label = ctk.CTkLabel(
            card,
            text=value,
            font=ctk.CTkFont(size=32, weight="bold"),
            text_color=color
        )
        value_label.pack(pady=(20, 5))
        
        label_widget = ctk.CTkLabel(
            card,
            text=label,
            font=ctk.CTkFont(size=11),
            text_color=TEXT_SECONDARY
        )
        label_widget.pack(pady=(0, 20))
        
        return card
    
    def _on_card_hover(self, card: ctk.CTkFrame, color: str):
        """Efecto hover en card"""
        card.configure(fg_color=color)
    
    def _create_feature_item(self, icon: str, title: str, desc: str):
        """Crear item de feature"""
        item = ctk.CTkFrame(self, fg_color=BG_CARD, corner_radius=8)
        item.pack(fill="x", padx=30, pady=6)
        
        # Bind hover
        item.bind("<Enter>", lambda e: item.configure(fg_color=BG_HOVER))
        item.bind("<Leave>", lambda e: item.configure(fg_color=BG_CARD))
        
        content = ctk.CTkFrame(item, fg_color="transparent")
        content.pack(fill="x", padx=20, pady=15)
        
        # Icon
        icon_label = ctk.CTkLabel(
            content,
            text=icon,
            font=ctk.CTkFont(size=24),
            width=40
        )
        icon_label.pack(side="left", padx=(0, 15))
        
        # Text container
        text_container = ctk.CTkFrame(content, fg_color="transparent")
        text_container.pack(side="left", fill="x", expand=True)
        
        title_label = ctk.CTkLabel(
            text_container,
            text=title,
            font=ctk.CTkFont(size=14, weight="bold"),
            anchor="w"
        )
        title_label.pack(fill="x")
        
        desc_label = ctk.CTkLabel(
            text_container,
            text=desc,
            font=ctk.CTkFont(size=11),
            text_color=TEXT_SECONDARY,
            anchor="w"
        )
        desc_label.pack(fill="x")
    
    def _create_tech_card(self, parent, name: str, category: str, color: str) -> ctk.CTkFrame:
        """Crear card de tecnología"""
        card = ctk.CTkFrame(parent, fg_color=BG_CARD, corner_radius=6)
        
        # Hover effect
        card.bind("<Enter>", lambda e: card.configure(fg_color=BG_HOVER))
        card.bind("<Leave>", lambda e: card.configure(fg_color=BG_CARD))
        
        # Category badge
        cat_label = ctk.CTkLabel(
            card,
            text=category,
            font=ctk.CTkFont(size=9),
            text_color=TEXT_DISABLED
        )
        cat_label.pack(pady=(12, 4))
        
        # Tech name
        name_label = ctk.CTkLabel(
            card,
            text=name,
            font=ctk.CTkFont(size=12, weight="bold"),
            text_color=color
        )
        name_label.pack(pady=(0, 12))
        
        return card
    
    def _create_link_button(self, parent, text: str, url: str, 
                           command: Optional[callable] = None, 
                           disabled: bool = False) -> ctk.CTkButton:
        """Crear botón de enlace"""
        from .theme import BORDER_DEFAULT, BORDER_HOVER
        
        btn = ctk.CTkButton(
            parent,
            text=text,
            font=ctk.CTkFont(size=13),
            height=40,
            corner_radius=8,
            fg_color=BORDER_DEFAULT if not disabled else BG_CARD,
            hover_color=BORDER_HOVER if not disabled else BG_CARD,
            command=command if not disabled else None,
            state="normal" if not disabled else "disabled"
        )
        
        return btn

/**
 * Alert Store - Sistema centralizado de alertas
 * Maneja toasts, errores detectados y notificaciones de Windows
 * By LOUST
 */
import { writable, derived } from 'svelte/store';

export interface Alert {
    id: string;
    type: 'success' | 'error' | 'warning' | 'info';
    title: string;
    message: string;
    timestamp: Date;
    duration?: number;
    source?: string; // 'network', 'dns', 'system', etc.
    dismissed?: boolean;
}

// Store principal de alertas
const alertsStore = writable<Alert[]>([]);

// Alertas visibles (no dismisseadas)
export const activeAlerts = derived(alertsStore, ($alerts) => 
    $alerts.filter(a => !a.dismissed)
);

// Errores para el panel de problemas
export const errorAlerts = derived(alertsStore, ($alerts) => 
    $alerts.filter(a => a.type === 'error')
);

// Contador de errores no vistos
export const errorCount = derived(errorAlerts, ($errors) => $errors.length);

// ID único para alertas
let alertId = 0;
function generateId(): string {
    return `alert-${Date.now()}-${++alertId}`;
}

// API pública
export const alerts = {
    subscribe: alertsStore.subscribe,
    
    /**
     * Añadir una nueva alerta
     */
    add(alert: Omit<Alert, 'id' | 'timestamp' | 'dismissed'>): string {
        const id = generateId();
        const newAlert: Alert = {
            ...alert,
            id,
            timestamp: new Date(),
            dismissed: false
        };
        
        alertsStore.update(alerts => [newAlert, ...alerts]);
        
        // Intentar mostrar notificación de Windows si es error
        if (alert.type === 'error') {
            showWindowsNotification(newAlert);
        }
        
        return id;
    },
    
    /**
     * Shortcuts para tipos comunes
     */
    success(title: string, message: string, source?: string) {
        return this.add({ type: 'success', title, message, source, duration: 4000 });
    },
    
    error(title: string, message: string, source?: string) {
        return this.add({ type: 'error', title, message, source, duration: 0 }); // Persistente
    },
    
    warning(title: string, message: string, source?: string) {
        return this.add({ type: 'warning', title, message, source, duration: 6000 });
    },
    
    info(title: string, message: string, source?: string) {
        return this.add({ type: 'info', title, message, source, duration: 5000 });
    },
    
    /**
     * Dismiss (ocultar del toast pero mantener en historial)
     */
    dismiss(id: string) {
        alertsStore.update(alerts => 
            alerts.map(a => a.id === id ? { ...a, dismissed: true } : a)
        );
    },
    
    /**
     * Eliminar completamente
     */
    remove(id: string) {
        alertsStore.update(alerts => alerts.filter(a => a.id !== id));
    },
    
    /**
     * Limpiar todas las alertas
     */
    clear() {
        alertsStore.set([]);
    },
    
    /**
     * Limpiar solo errores
     */
    clearErrors() {
        alertsStore.update(alerts => alerts.filter(a => a.type !== 'error'));
    }
};

/**
 * Intentar mostrar notificación de Windows via Tauri
 */
async function showWindowsNotification(alert: Alert) {
    try {
        if (typeof window === 'undefined' || !('__TAURI_INTERNALS__' in window)) {
            return;
        }

        const { sendNotification, isPermissionGranted, requestPermission } = await import('@tauri-apps/plugin-notification');
        
        let permitted = await isPermissionGranted();
        if (!permitted) {
            const permission = await requestPermission();
            permitted = permission === 'granted';
        }
        
        if (permitted) {
            await sendNotification({
                title: `NetBoozt: ${alert.title}`,
                body: alert.message
            });
        }
    } catch (e) {
        // Silenciosamente fallar si no está disponible
        console.debug('Windows notification not available:', e);
    }
}

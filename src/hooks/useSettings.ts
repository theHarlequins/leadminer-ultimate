import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";

export interface Settings {
    api_key: string;
    model_id: string;
}

export const useSettings = () => {
    const [settings, setSettings] = useState<Settings>({
        api_key: "",
        model_id: "google/gemini-2.0-flash-exp:free"
    });
    const [loading, setLoading] = useState(true);

    const loadSettings = async () => {
        try {
            const loaded = await invoke<Settings>("get_settings");
            setSettings(loaded);
        } catch (e) {
            console.error("Failed to load settings:", e);
        } finally {
            setLoading(false);
        }
    };

    const saveSettings = async (newSettings: Settings) => {
        try {
            await invoke("save_settings", { settings: newSettings });
            setSettings(newSettings);
            return true;
        } catch (e) {
            console.error("Failed to save settings:", e);
            return false;
        }
    };

    useEffect(() => {
        loadSettings();
    }, []);

    return {
        settings,
        loading,
        saveSettings,
        loadSettings
    };
};

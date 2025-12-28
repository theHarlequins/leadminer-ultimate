import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useMutation, useQueryClient } from '@tanstack/react-query'; // Check if react-query is available here, if not need to pass props or setup context
import { Settings as SettingsIcon, Save, RefreshCw, Key, Shield, AlertCircle, CheckCircle } from "lucide-react";

interface Model {
    id: string;
    name: string;
    pricing: {
        prompt: string;
        completion: string;
    };
}

export default function Settings() {
    const [apiKey, setApiKey] = useState("");
    const [models, setModels] = useState<Model[]>([]);
    const [showFreeOnly, setShowFreeOnly] = useState(true);
    const [loading, setLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);

    // Proxy State
    const [proxies, setProxies] = useState<string[]>([]);
    const queryClient = useQueryClient();

    const updateProxiesMutation = useMutation({
        mutationFn: async (proxyList: string[]) => {
            await invoke('update_proxies', { proxies: proxyList });
        },
        onSuccess: () => {
            queryClient.invalidateQueries({ queryKey: ['proxies'] });
        },
    });

    // Load saved settings on mount
    useEffect(() => {
        // TODO: Load API key from backend/store
    }, []);

    const fetchModels = async () => {
        setLoading(true);
        setError(null);
        try {
            // In a real app, this would call the Rust command `fetch_models`
            // treating apiKey as potential auth header if needed or relying on backend stored key
            // For now, we'll simulate a fetch or call a placeholder
            // const res = await invoke<Model[]>("fetch_ai_models");

            // Mock data for demonstration until backend is wired up completely
            const mockModels: Model[] = [
                { id: "gpt-4", name: "GPT-4", pricing: { prompt: "0.03", completion: "0.06" } },
                { id: "claude-3-opus", name: "Claude 3 Opus", pricing: { prompt: "0.015", completion: "0.075" } },
                { id: "mistral-7b-free", name: "Mistral 7B (Free)", pricing: { prompt: "0", completion: "0" } },
                { id: "llama-3-8b-free", name: "Llama 3 8B (Free)", pricing: { prompt: "0", completion: "0" } },
            ];

            // Simulate delay
            await new Promise(r => setTimeout(r, 1000));
            setModels(mockModels);

        } catch (err) {
            setError("Failed to fetch models. Check your API key.");
            console.error(err);
        } finally {
            setLoading(false);
        }
    };

    const filteredModels = showFreeOnly
        ? models.filter(m => m.pricing.prompt === "0" && m.pricing.completion === "0")
        : models;

    return (
        <div className="p-8 bg-[#e0e5ec] text-gray-700 min-h-screen font-sans">
            <div className="max-w-3xl mx-auto">

                {/* Header */}
                <div className="flex items-center gap-4 mb-8">
                    <div className="p-4 rounded-full bg-[#e0e5ec] shadow-[6px_6px_12px_#b8b9be,-6px_-6px_12px_#ffffff]">
                        <SettingsIcon className="w-8 h-8 text-blue-500" />
                    </div>
                    <h1 className="text-3xl font-bold tracking-tight text-gray-800">Settings</h1>
                </div>

                {/* API Key Section */}
                <div className="mb-8 p-8 rounded-3xl bg-[#e0e5ec] shadow-[inset_6px_6px_12px_#b8b9be,inset_-6px_-6px_12px_#ffffff]">
                    <h2 className="flex items-center gap-2 text-xl font-semibold mb-6">
                        <Key className="w-5 h-5 text-gray-500" /> OpenRouter API Key
                    </h2>

                    <div className="flex gap-4">
                        <input
                            type="password"
                            value={apiKey}
                            onChange={(e) => setApiKey(e.target.value)}
                            placeholder="sk-or-..."
                            className="flex-1 p-4 rounded-xl bg-[#e0e5ec] shadow-[inset_4px_4px_8px_#b8b9be,inset_-4px_-4px_8px_#ffffff] outline-none focus:shadow-[inset_2px_2px_4px_#b8b9be,inset_-2px_-2px_4px_#ffffff] transition-shadow"
                        />
                        <button
                            className="px-6 py-3 rounded-xl font-semibold text-blue-600 bg-[#e0e5ec] shadow-[6px_6px_12px_#b8b9be,-6px_-6px_12px_#ffffff] hover:shadow-[inset_4px_4px_8px_#b8b9be,inset_-4px_-4px_8px_#ffffff] active:translate-y-[1px] transition-all flex items-center gap-2"
                        >
                            <Save className="w-5 h-5" /> Save
                        </button>
                    </div>
                </div>

                {/* Proxy Section */}
                <div className="mb-8 p-8 rounded-3xl bg-[#e0e5ec] shadow-[9px_9px_16px_#b8b9be,-9px_-9px_16px_#ffffff]">
                    <h2 className="flex items-center gap-2 text-xl font-semibold mb-6">
                        <SettingsIcon className="w-5 h-5 text-gray-500" /> Proxy Settings
                    </h2>
                    <textarea
                        placeholder="Enter proxies (one per line)&#10;Format: http://user:pass@host:port&#10;Or: socks5://host:port"
                        value={proxies.join('\n')}
                        onChange={(e) => setProxies(e.target.value.split('\n'))}
                        className="w-full h-32 p-4 rounded-xl bg-[#e0e5ec] shadow-[inset_4px_4px_8px_#b8b9be,inset_-4px_-4px_8px_#ffffff] outline-none focus:shadow-[inset_2px_2px_4px_#b8b9be,inset_-2px_-2px_4px_#ffffff] transition-shadow font-mono text-sm mb-4"
                    />
                    <div className="flex items-center gap-4">
                        <button
                            onClick={() => updateProxiesMutation.mutate(proxies.filter(Boolean))} // Filter empty lines
                            disabled={updateProxiesMutation.isPending}
                            className="px-6 py-3 rounded-xl font-semibold text-blue-600 bg-[#e0e5ec] shadow-[6px_6px_12px_#b8b9be,-6px_-6px_12px_#ffffff] hover:shadow-[inset_4px_4px_8px_#b8b9be,inset_-4px_-4px_8px_#ffffff] active:translate-y-[1px] transition-all flex items-center gap-2"
                        >
                            <Save className="w-5 h-5" />
                            {updateProxiesMutation.isPending ? 'Saving...' : `Save Proxies (${proxies.filter(Boolean).length})`}
                        </button>
                        {updateProxiesMutation.isSuccess && (
                            <span className="text-green-600 flex items-center gap-1">
                                <CheckCircle size={16} /> Saved!
                            </span>
                        )}
                    </div>
                </div>

                {/* Models Section */}
                <div className="p-8 rounded-3xl bg-[#e0e5ec] shadow-[9px_9px_16px_#b8b9be,-9px_-9px_16px_#ffffff]">
                    <div className="flex justify-between items-center mb-6">
                        <h2 className="flex items-center gap-2 text-xl font-semibold">
                            <Shield className="w-5 h-5 text-gray-500" /> AI Models
                        </h2>

                        <button
                            onClick={fetchModels}
                            disabled={loading}
                            className={`p-3 rounded-full bg-[#e0e5ec] shadow-[6px_6px_12px_#b8b9be,-6px_-6px_12px_#ffffff] hover:shadow-[4px_4px_8px_#b8b9be,-4px_-4px_8px_#ffffff] active:shadow-[inset_4px_4px_8px_#b8b9be,inset_-4px_-4px_8px_#ffffff] transition-all ${loading ? 'opacity-50' : ''}`}
                        >
                            <RefreshCw className={`w-6 h-6 text-blue-500 ${loading ? 'animate-spin' : ''}`} />
                        </button>
                    </div>

                    {error && (
                        <div className="mb-4 p-4 rounded-xl bg-red-100 text-red-600 flex items-center gap-2 shadow-[inset_2px_2px_5px_rgba(0,0,0,0.1)]">
                            <AlertCircle className="w-5 h-5" /> {error}
                        </div>
                    )}

                    {/* Toggle Switch */}
                    <div className="flex items-center gap-3 mb-6 pl-2">
                        <div
                            onClick={() => setShowFreeOnly(!showFreeOnly)}
                            className={`relative w-14 h-8 rounded-full cursor-pointer transition-colors duration-300 shadow-[inset_3px_3px_6px_#b8b9be,inset_-3px_-3px_6px_#ffffff] ${showFreeOnly ? 'bg-blue-400' : 'bg-[#e0e5ec]'}`}
                        >
                            <div className={`absolute top-1 left-1 w-6 h-6 rounded-full bg-[#e0e5ec] shadow-[2px_2px_5px_rgba(0,0,0,0.2)] transform transition-transform duration-300 ${showFreeOnly ? 'translate-x-6' : 'translate-x-0'}`}></div>
                        </div>
                        <span className="font-medium text-gray-600">Show Free Models Only</span>
                    </div>

                    {/* Models List */}
                    <div className="space-y-4 max-h-[400px] overflow-y-auto pr-2 custom-scrollbar">
                        {models.length === 0 && !loading && (
                            <p className="text-gray-400 text-center italic py-8">Click refresh to load models...</p>
                        )}

                        {filteredModels.map((model) => (
                            <div key={model.id} className="p-4 rounded-xl bg-[#e0e5ec] shadow-[4px_4px_8px_#b8b9be,-4px_-4px_8px_#ffffff] flex justify-between items-center group hover:bg-gray-50 transition-colors">
                                <div>
                                    <h3 className="font-bold text-gray-800">{model.name}</h3>
                                    <code className="text-xs text-gray-500 bg-gray-200 px-2 py-1 rounded-md">{model.id}</code>
                                </div>
                                <div className="text-right">
                                    <div className={`text-sm font-semibold ${model.pricing.prompt === "0" ? 'text-green-600' : 'text-gray-600'}`}>
                                        {model.pricing.prompt === "0" ? "FREE" : `$${model.pricing.prompt} / 1M`}
                                    </div>
                                    {model.pricing.prompt !== "0" && (
                                        <div className="text-xs text-gray-400">Prompt</div>
                                    )}
                                </div>
                            </div>
                        ))}
                    </div>
                </div>

            </div>
        </div>
    );
}

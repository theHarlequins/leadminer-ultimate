import { useState, Fragment, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Combobox, Switch, Transition } from "@headlessui/react";
import { Eye, EyeOff, RefreshCw, Check, Save } from "lucide-react";
import clsx from "clsx";
import { useSettings } from "../hooks/useSettings";

interface Pricing {
    prompt: string;
    completion: string;
}

interface Model {
    id: string;
    name: string;
    pricing: Pricing;
}

export default function AiSettingsPanel() {
    const { settings, saveSettings, loading: settingsLoading } = useSettings();
    const [apiKey, setApiKey] = useState("");
    const [showKey, setShowKey] = useState(false);
    const [models, setModels] = useState<Model[]>([]);
    const [selectedModel, setSelectedModel] = useState<string>("");
    const [query, setQuery] = useState("");
    const [loading, setLoading] = useState(false);
    const [freeOnly, setFreeOnly] = useState(true);
    const [savedSuccess, setSavedSuccess] = useState(false);

    useEffect(() => {
        if (!settingsLoading) {
            setApiKey(settings.api_key);
            setSelectedModel(settings.model_id);
        }
    }, [settings, settingsLoading]);

    const handleSave = async () => {
        const success = await saveSettings({
            api_key: apiKey,
            model_id: selectedModel
        });
        if (success) {
            setSavedSuccess(true);
            setTimeout(() => setSavedSuccess(false), 2000);
        }
    };

    // Use a map to handle custom inputs + fetched models
    const allModels = query === ""
        ? models
        : models.filter((model) =>
            model.name.toLowerCase().includes(query.toLowerCase()) ||
            model.id.toLowerCase().includes(query.toLowerCase())
        );

    // If query is non-empty and not in list, we treat it as a custom "new" model
    // but ComboBox allows custom values by default if we just handle the onChange string.

    const filteredModels = freeOnly
        ? allModels.filter(m => m.pricing.prompt === "0")
        : allModels;

    const fetchModels = async () => {
        if (!apiKey) return;
        setLoading(true);
        try {
            const fetched: Model[] = await invoke("fetch_models", { apiKey });
            setModels(fetched);
        } catch (e) {
            console.error("Failed to fetch models:", e);
        } finally {
            setLoading(false);
        }
    };

    return (
        <div className="p-8 rounded-3xl bg-[#e0e5ec] shadow-[9px_9px_16px_#b8b9be,-9px_-9px_16px_#ffffff] space-y-8">
            {/* Header */}
            <h2 className="text-2xl font-bold text-gray-700 tracking-tight">AI Power Configuration</h2>

            {/* API Key Section */}
            <div className="space-y-3">
                <label className="block text-sm font-semibold text-gray-600 ml-1">OpenRouter API Key</label>
                <div className="flex gap-4">
                    <div className="relative flex-1 group">
                        <input
                            type={showKey ? "text" : "password"}
                            value={apiKey}
                            onChange={(e) => setApiKey(e.target.value)}
                            placeholder="sk-or-..."
                            className="w-full p-4 pr-12 rounded-xl bg-[#e0e5ec] shadow-[inset_3px_3px_6px_#b8b9be,inset_-3px_-3px_6px_#ffffff] focus:outline-none focus:shadow-[inset_2px_2px_4px_#b8b9be,inset_-2px_-2px_4px_#ffffff] transition-shadow text-gray-700 font-mono tracking-wide"
                        />
                        <button
                            onClick={() => setShowKey(!showKey)}
                            className="absolute right-4 top-1/2 -translate-y-1/2 text-gray-400 hover:text-gray-600 transition-colors"
                        >
                            {showKey ? <EyeOff size={20} /> : <Eye size={20} />}
                        </button>
                    </div>

                    <button
                        onClick={fetchModels}
                        disabled={!apiKey || loading}
                        className={clsx(
                            "px-4 rounded-xl flex items-center justify-center transition-all bg-[#e0e5ec]",
                            "shadow-[5px_5px_10px_#b8b9be,-5px_-5px_10px_#ffffff]",
                            "hover:shadow-[3px_3px_6px_#b8b9be,-3px_-3px_6px_#ffffff] hover:-translate-y-[1px]",
                            "active:shadow-[inset_3px_3px_6px_#b8b9be,inset_-3px_-3px_6px_#ffffff] active:translate-y-0",
                            loading && "animate-pulse"
                        )}
                        title="Fetch Models"
                    >
                        <RefreshCw size={24} className={clsx("text-blue-500", loading && "animate-spin")} />
                    </button>
                </div>
            </div>

            {/* Model Selector & Filters */}
            <div className="space-y-1">
                <div className="flex justify-between items-center mb-2">
                    <label className="block text-sm font-semibold text-gray-600 ml-1">AI Model</label>

                    {/* Free Only Toggle */}
                    <Switch.Group>
                        <div className="flex items-center gap-3">
                            <Switch.Label className="text-xs font-medium text-gray-500">Free Models Only</Switch.Label>
                            <Switch
                                checked={freeOnly}
                                onChange={setFreeOnly}
                                className={clsx(
                                    "relative inline-flex h-6 w-11 items-center rounded-full transition-colors focus:outline-none",
                                    freeOnly ? "bg-green-400" : "bg-gray-300",
                                    "shadow-[inset_2px_2px_4px_rgba(0,0,0,0.1)]"
                                )}
                            >
                                <span
                                    className={clsx(
                                        "inline-block h-4 w-4 transform rounded-full bg-white transition-transform shadow-sm",
                                        freeOnly ? "translate-x-6" : "translate-x-1"
                                    )}
                                />
                            </Switch>
                        </div>
                    </Switch.Group>
                </div>

                <Combobox value={selectedModel} onChange={(val) => setSelectedModel(val || "")}>
                    <div className="relative">
                        <div className="relative">
                            <Combobox.Input
                                onChange={(event) => setQuery(event.target.value)}
                                displayValue={(item: string) => item} // Display the raw string ID
                                className="w-full p-4 rounded-xl bg-[#e0e5ec] shadow-[inset_3px_3px_6px_#b8b9be,inset_-3px_-3px_6px_#ffffff] focus:outline-none focus:shadow-[inset_2px_2px_4px_#b8b9be,inset_-2px_-2px_4px_#ffffff] transition-shadow text-gray-700 font-medium"
                                placeholder="Select or type model ID..."
                            />
                        </div>
                        <Transition
                            as={Fragment}
                            leave="transition ease-in duration-100"
                            leaveFrom="opacity-100"
                            leaveTo="opacity-0"
                            afterLeave={() => setQuery('')}
                        >
                            <Combobox.Options className="absolute z-10 w-full mt-2 max-h-60 overflow-auto rounded-xl bg-[#e0e5ec] py-2 shadow-[9px_9px_16px_rgba(163,177,198,0.6),-9px_-9px_16px_rgba(255,255,255,0.8)] focus:outline-none sm:text-sm custom-scrollbar">
                                {query.length > 0 && !filteredModels.some(m => m.id === query) && (
                                    <Combobox.Option
                                        value={query}
                                        className={({ active }) =>
                                            clsx(
                                                "relative cursor-default select-none py-3 px-4",
                                                active ? "bg-blue-100 text-blue-900" : "text-gray-900"
                                            )
                                        }
                                    >
                                        <span className="block truncate">Use custom: "{query}"</span>
                                    </Combobox.Option>
                                )}

                                {filteredModels.map((model) => (
                                    <Combobox.Option
                                        key={model.id}
                                        value={model.id}
                                        className={({ active }) =>
                                            clsx(
                                                "relative cursor-default select-none py-3 px-4 flex justify-between items-center",
                                                active ? "bg-blue-50" : "hover:bg-gray-100"
                                            )
                                        }
                                    >
                                        {({ selected }) => (
                                            <>
                                                <div className="flex items-center gap-2">
                                                    {selected && <Check size={16} className="text-blue-500" />}
                                                    <div>
                                                        <span className={clsx("block truncate font-medium", selected ? "text-blue-600" : "text-gray-700")}>
                                                            {model.name}
                                                        </span>
                                                        <span className="text-xs text-xs text-gray-400 font-mono">{model.id}</span>
                                                    </div>
                                                </div>

                                                {model.pricing.prompt === "0" ? (
                                                    <span className="px-2 py-1 rounded text-[10px] font-bold bg-green-100 text-green-700 shadow-sm border border-green-200">
                                                        FREE
                                                    </span>
                                                ) : (
                                                    <span className="px-2 py-1 rounded text-[10px] font-bold bg-gray-200 text-gray-600 shadow-sm">
                                                        $
                                                    </span>
                                                )}
                                            </>
                                        )}
                                    </Combobox.Option>
                                ))}
                            </Combobox.Options>
                        </Transition>
                    </div>
                </Combobox>
            </div>

            {/* Save Button */}
            {/* Save Button */}
            <button
                onClick={handleSave}
                className={clsx(
                    "w-full py-4 rounded-xl font-bold tracking-wide transition-all flex items-center justify-center gap-2",
                    "bg-[#e0e5ec] shadow-[6px_6px_12px_#b8b9be,-6px_-6px_12px_#ffffff]",
                    "hover:shadow-[4px_4px_8px_#b8b9be,-4px_-4px_8px_#ffffff] hover:text-blue-700",
                    "active:shadow-[inset_4px_4px_8px_#b8b9be,inset_-4px_-4px_8px_#ffffff] active:translate-y-[1px]",
                    savedSuccess ? "text-green-600" : "text-blue-600"
                )}>
                {savedSuccess ? <Check size={20} /> : <Save size={20} />}
                {savedSuccess ? "Saved Successfully" : "Save Configuration"}
            </button>

        </div>
    );
}

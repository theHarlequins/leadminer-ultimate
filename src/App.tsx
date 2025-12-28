import React, { useState } from 'react';
import { useQuery, useQueryClient } from '@tanstack/react-query';
import { invoke } from '@tauri-apps/api/core';
import {
  Copy,
  MessageCircle,
  MessageSquare,
  AlertCircle,
  Search,
  Settings,
  Download,
  Wifi,
  AlertTriangle
} from 'lucide-react';
import AiSettingsPanel from './components/AiSettingsPanel';
import { useSettings } from './hooks/useSettings';

interface Lead {
  name: string;
  address: string;
  phone: string;
  normalized_phone: string;
  phone_type: 'Mobile' | 'Landline' | 'Unknown';
  website?: string;
  instagram?: string;
  facebook?: string;
  city: string;
  status: 'New' | 'Contacted' | 'BadLead';
}

const App: React.FC = () => {
  const queryClient = useQueryClient();
  const { settings, loading: settingsLoading } = useSettings();
  const [city, setCity] = useState('');
  const [query, setQuery] = useState('обувь');
  const [showSettings, setShowSettings] = useState(false);
  const [showConnectionTest, setShowConnectionTest] = useState(false);
  const [copiedPhone, setCopiedPhone] = useState<string | null>(null);
  // Запрос для получения лидов
  const leadsQuery = useQuery<Lead[]>({
    queryKey: ['leads', settings.api_key],
    queryFn: async () => {
      console.log('[DEBUG] Search triggered with settings:', {
        apiKey: settings.api_key ? 'PROVIDED' : 'EMPTY',
        modelId: settings.model_id
      });
      const result = await invoke<Lead[]>('start_scraping', {
        city,
        query,
        apiKey: settings.api_key,
        modelId: settings.model_id
      });
      return result;
    },
    enabled: false,
    retry: 2,
  });

  // Запрос для тестирования соединения
  const connectionTestQuery = useQuery<string>({
    queryKey: ['connection'],
    queryFn: async () => {
      const result = await invoke<string>('test_connection');
      return result;
    },
    enabled: false,
    retry: 1,
  });

  // Копирование телефона
  const copyPhone = async (phone: string) => {
    try {
      await navigator.clipboard.writeText(phone);
      setCopiedPhone(phone);
      setTimeout(() => setCopiedPhone(null), 2000);
    } catch (err) {
      console.error('Failed to copy:', err);
    }
  };

  // Открыть в Viber
  const openViber = (phone: string) => {
    window.open(`viber://chat?number=${phone}`, '_blank');
  };

  // Открыть в Telegram
  const openTelegram = (phone: string) => {
    window.open(`tg://resolve?domain=${phone}`, '_blank');
  };

  // Скачать результаты
  const downloadResults = () => {
    if (!leadsQuery.data) return;

    const csv = convertToCSV(leadsQuery.data);
    const blob = new Blob([csv], { type: 'text/csv' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `leadminer_${city}_${new Date().toISOString().split('T')[0]}.csv`;
    a.click();
    URL.revokeObjectURL(url);
  };

  // Конвертация в CSV
  const convertToCSV = (data: Lead[]): string => {
    const headers = ['Name', 'City', 'Phone', 'Normalized', 'Type', 'Website', 'Instagram', 'Facebook', 'Status'];
    const rows = data.map(lead => [
      lead.name,
      lead.city,
      lead.phone,
      lead.normalized_phone,
      lead.phone_type,
      lead.website || '',
      lead.instagram || '',
      lead.facebook || '',
      lead.status
    ]);

    return [headers, ...rows].map(row => row.map(cell => `"${cell}"`).join(',')).join('\n');
  };

  // Запуск теста соединения
  const runConnectionTest = () => {
    setShowConnectionTest(true);
    connectionTestQuery.refetch();
  };

  // Стили Neumorphism
  const neumorphicCard = "bg-gray-100 rounded-xl p-6 shadow-neumorphism";
  const neumorphicInput = "bg-gray-100 rounded-lg px-4 py-2 shadow-neumorphism-inset focus:outline-none focus:ring-2 focus:ring-blue-500";
  const neumorphicButton = "bg-gray-100 rounded-lg px-6 py-3 shadow-neumorphism hover:shadow-neumorphism-inset transition-all duration-200 font-medium";
  const neumorphicBadge = "px-3 py-1 rounded-full text-xs font-semibold";

  return (
    <div className="min-h-screen bg-gray-50 p-8 font-sans">
      <div className="max-w-7xl mx-auto">
        {/* Header */}
        <header className="mb-8">
          <h1 className="text-4xl font-bold text-gray-800 mb-2">LeadMiner Ultimate</h1>
          <p className="text-gray-600">B2B Lead Mining & OSINT Tool</p>
        </header>

        {/* Controls */}
        <div className={`${neumorphicCard} mb-6`}>
          <div className="grid grid-cols-1 md:grid-cols-4 gap-4 items-end">
            <div>
              <label className="block text-sm font-medium text-gray-700 mb-2">Город</label>
              <input
                type="text"
                value={city}
                onChange={(e) => setCity(e.target.value)}
                placeholder="Киев, Львов..."
                className={`${neumorphicInput} w-full`}
              />
            </div>

            <div>
              <label className="block text-sm font-medium text-gray-700 mb-2">Запрос</label>
              <input
                type="text"
                value={query}
                onChange={(e) => setQuery(e.target.value)}
                placeholder="обувь, магазин..."
                className={`${neumorphicInput} w-full`}
              />
            </div>

            <div className="flex gap-2">
              <button
                onClick={() => leadsQuery.refetch()}
                disabled={!city || leadsQuery.isFetching}
                className={`${neumorphicButton} flex-1 flex items-center justify-center gap-2 ${leadsQuery.isFetching ? 'opacity-50 cursor-not-allowed' : ''
                  }`}
              >
                <Search size={18} />
                {leadsQuery.isFetching ? 'Поиск...' : 'Искать'}
              </button>
            </div>

            <div className="flex gap-2">
              <button
                onClick={() => setShowSettings(!showSettings)}
                className={`${neumorphicButton} flex-1 flex items-center justify-center gap-2`}
              >
                <Settings size={18} />
                Прокси
              </button>
              <button
                onClick={downloadResults}
                disabled={!leadsQuery.data}
                className={`${neumorphicButton} flex-1 flex items-center justify-center gap-2 ${!leadsQuery.data ? 'opacity-50 cursor-not-allowed' : ''
                  }`}
              >
                <Download size={18} />
              </button>
            </div>
          </div>

          {/* Connection Test Button */}
          <div className="mt-4 flex gap-2">
            <button
              onClick={runConnectionTest}
              disabled={connectionTestQuery.isFetching}
              className={`${neumorphicButton} flex items-center justify-center gap-2 ${connectionTestQuery.isFetching ? 'opacity-50 cursor-not-allowed' : ''
                }`}
            >
              <Wifi size={18} />
              {connectionTestQuery.isFetching ? 'Тестирование...' : 'Проверить соединение'}
            </button>
          </div>

          {/* Connection Test Results */}
          {showConnectionTest && connectionTestQuery.data && (
            <div className="mt-6 pt-6 border-t border-gray-200">
              <h3 className="text-lg font-semibold text-gray-800 mb-3">Результаты теста</h3>
              <div className={`${neumorphicCard} bg-white`}>
                <pre className="text-sm text-gray-700 whitespace-pre-wrap font-mono">
                  {connectionTestQuery.data}
                </pre>
              </div>
              <button
                onClick={() => {
                  setShowConnectionTest(false);
                  queryClient.removeQueries({ queryKey: ['connection'] });
                }}
                className="mt-2 text-sm text-gray-600 hover:text-gray-800 underline"
              >
                Скрыть результаты
              </button>
            </div>
          )}

          {/* Connection Test Error */}
          {showConnectionTest && connectionTestQuery.error && (
            <div className="mt-6 pt-6 border-t border-gray-200">
              <div className={`${neumorphicCard} bg-red-50 border border-red-200`}>
                <div className="flex items-center gap-2 text-red-700">
                  <AlertTriangle size={20} />
                  <span>Ошибка теста: {connectionTestQuery.error.message}</span>
                </div>
              </div>
            </div>
          )}

          {/* Proxy Settings */}
          {showSettings && (
            <div className="mt-6 pt-6 border-t border-gray-200">
              <AiSettingsPanel />
            </div>
          )}
        </div>

        {/* Error Display */}
        {leadsQuery.error && (
          <div className={`${neumorphicCard} mb-6 bg-red-50 border border-red-200`}>
            <div className="flex items-center gap-2 text-red-700">
              <AlertCircle size={20} />
              <span>Ошибка: {leadsQuery.error.message}</span>
            </div>
          </div>
        )}

        {/* Results Table */}
        {leadsQuery.data && leadsQuery.data.length > 0 && (
          <div className={`${neumorphicCard} overflow-hidden`}>
            <div className="flex justify-between items-center mb-4">
              <h2 className="text-xl font-semibold text-gray-800">
                Найдено лидов: {leadsQuery.data.length}
              </h2>
              <div className="flex gap-2 text-sm">
                <span className="flex items-center gap-1">
                  <span className="w-3 h-3 rounded-full bg-green-500"></span>
                  Мобильные
                </span>
                <span className="flex items-center gap-1">
                  <span className="w-3 h-3 rounded-full bg-gray-400"></span>
                  Стационарные
                </span>
              </div>
            </div>

            <div className="overflow-x-auto">
              <table className="w-full text-sm">
                <thead>
                  <tr className="border-b-2 border-gray-200">
                    <th className="text-left py-3 px-4 font-semibold text-gray-700">Название</th>
                    <th className="text-left py-3 px-4 font-semibold text-gray-700">Город</th>
                    <th className="text-left py-3 px-4 font-semibold text-gray-700">Телефон</th>
                    <th className="text-left py-3 px-4 font-semibold text-gray-700">Соцсети</th>
                    <th className="text-left py-3 px-4 font-semibold text-gray-700">Статус</th>
                    <th className="text-left py-3 px-4 font-semibold text-gray-700">Действия</th>
                  </tr>
                </thead>
                <tbody>
                  {leadsQuery.data.map((lead, index) => (
                    <tr
                      key={index}
                      className="border-b border-gray-100 hover:bg-gray-50 transition-colors"
                    >
                      <td className="py-3 px-4 font-medium text-gray-800">{lead.name}</td>
                      <td className="py-3 px-4 text-gray-600">{lead.city}</td>
                      <td className="py-3 px-4">
                        <div className="flex items-center gap-2">
                          <span
                            className={`font-mono font-semibold ${lead.phone_type === 'Mobile' ? 'text-green-600' : 'text-gray-500'
                              }`}
                          >
                            {lead.normalized_phone}
                          </span>
                          <button
                            onClick={() => copyPhone(lead.normalized_phone)}
                            className="p-1 hover:bg-gray-200 rounded transition-colors"
                            title="Копировать"
                          >
                            <Copy size={14} className={copiedPhone === lead.normalized_phone ? 'text-green-600' : 'text-gray-500'} />
                          </button>
                        </div>
                      </td>
                      <td className="py-3 px-4">
                        <div className="flex gap-2">
                          {lead.instagram && (
                            <span className="text-pink-600 font-medium text-xs">
                              IG: {lead.instagram}
                            </span>
                          )}
                          {lead.facebook && (
                            <span className="text-blue-600 font-medium text-xs">
                              FB
                            </span>
                          )}
                        </div>
                      </td>
                      <td className="py-3 px-4">
                        <span className={`${neumorphicBadge} ${lead.status === 'New' ? 'bg-blue-100 text-blue-700' :
                          lead.status === 'Contacted' ? 'bg-green-100 text-green-700' :
                            'bg-red-100 text-red-700'
                          }`}>
                          {lead.status === 'New' ? 'Новый' :
                            lead.status === 'Contacted' ? 'Контактирован' : 'Плохой'}
                        </span>
                      </td>
                      <td className="py-3 px-4">
                        <div className="flex gap-2">
                          {lead.phone_type === 'Mobile' && (
                            <>
                              <button
                                onClick={() => openViber(lead.normalized_phone)}
                                className="p-2 bg-purple-100 hover:bg-purple-200 rounded-lg transition-colors text-purple-700"
                                title="Открыть в Viber"
                              >
                                <MessageCircle size={16} />
                              </button>
                              <button
                                onClick={() => openTelegram(lead.normalized_phone)}
                                className="p-2 bg-blue-100 hover:bg-blue-200 rounded-lg transition-colors text-blue-700"
                                title="Открыть в Telegram"
                              >
                                <MessageSquare size={16} />
                              </button>
                            </>
                          )}
                          {lead.website && (
                            <a
                              href={lead.website}
                              target="_blank"
                              rel="noopener noreferrer"
                              className="p-2 bg-gray-100 hover:bg-gray-200 rounded-lg transition-colors text-gray-700"
                              title="Открыть сайт"
                            >
                              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
                                <path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"></path>
                                <polyline points="15 3 21 3 21 9"></polyline>
                                <line x1="10" y1="14" x2="21" y2="3"></line>
                              </svg>
                            </a>
                          )}
                        </div>
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          </div>
        )}

        {/* Empty State */}
        {!leadsQuery.data && !leadsQuery.isFetching && !leadsQuery.error && (
          <div className={`${neumorphicCard} text-center py-12`}>
            <div className="text-gray-400 mb-3">
              <Search size={48} className="mx-auto opacity-50" />
            </div>
            <p className="text-gray-600">Введите город и нажмите "Искать" для начала работы</p>
          </div>
        )}

        {/* Loading State */}
        {leadsQuery.isFetching && (
          <div className={`${neumorphicCard} text-center py-12`}>
            <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600 mx-auto mb-4"></div>
            <p className="text-gray-600">Идет поиск лидов...</p>
            <p className="text-sm text-gray-500 mt-2">Это может занять несколько минут</p>
          </div>
        )}
      </div>
    </div>
  );
};

export default App;
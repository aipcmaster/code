// 主窗口：左侧导航 + 内容区（四页：仪表盘/诊断/优化/设置）。
// 启动即触发快速诊断（对齐 PRD US-02：一键诊断 <3s）。

using System.Windows;
using AIPCMaster.App.Services;
using AIPCMaster.App.ViewModels;
using AIPCMaster.App.Views;
using AIPCMaster.Core.Collectors;

namespace AIPCMaster.App;

public partial class MainWindow : Window
{
    private readonly MainViewModel _vm;

    public MainWindow()
    {
        InitializeComponent();

        var settings = LocalSettings.Load();
        _vm = new MainViewModel(
            new WindowsSystemCollector(),
            settings,
            new ApiClient(settings.ApiBaseUrl));

        DataContext = _vm;

        Loaded += OnLoaded;
    }

    private void OnLoaded(object sender, RoutedEventArgs e)
    {
        Nav_Dashboard_Click(sender, e);
    }

    private void Nav_Dashboard_Click(object sender, RoutedEventArgs e)
    {
        ContentHost.Content = new DashboardView { DataContext = _vm };
        _vm.RefreshDashboardAsync().ConfigureAwait(false);
    }

    private void Nav_Diagnosis_Click(object sender, RoutedEventArgs e)
    {
        ContentHost.Content = new DiagnosisView { DataContext = _vm };
        _vm.RefreshDashboardAsync().ConfigureAwait(false);
    }

    private void Nav_Optimization_Click(object sender, RoutedEventArgs e)
    {
        ContentHost.Content = new OptimizationView { DataContext = _vm };
    }

    private void Nav_Settings_Click(object sender, RoutedEventArgs e)
    {
        ContentHost.Content = new SettingsView { DataContext = _vm };
        _vm.OnNavigatedToSettings();
    }
}
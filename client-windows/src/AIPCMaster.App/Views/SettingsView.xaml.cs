using System.Windows;
using System.Windows.Controls;

namespace AIPCMaster.App.Views;

public partial class SettingsView : UserControl
{
    public SettingsView() => InitializeComponent();

    /// 登录/注册前把 PasswordBox 值同步进 VM（避免 WPF 密码框无法绑定的限制）。
    private void SyncPassword()
    {
        if (DataContext is ViewModels.MainViewModel vm)
        {
            vm.LoginPassword = PasswordInput.Password;
        }
    }

    private async void Login_Click(object sender, RoutedEventArgs e)
    {
        SyncPassword();
        if (DataContext is ViewModels.MainViewModel vm)
        {
            await vm.LoginAsync();
        }
    }

    private async void Register_Click(object sender, RoutedEventArgs e)
    {
        SyncPassword();
        if (DataContext is ViewModels.MainViewModel vm)
        {
            await vm.RegisterAsync();
        }
    }
}
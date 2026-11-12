# Deploying Optimised PDF {{VERSION}} with Microsoft Intune

These files come from one run of **Actions > Build installer**:

| File | What it is |
|---|---|
| `{{MSI}}` | Windows installer (x64). Installed with `ALLUSERS=1` it is per-machine, for every user on the device |
| `OptimisedPDF-{{VERSION}}-x64.intunewin` | The same MSI, wrapped for an Intune **Windows app (Win32)** |
| `OptimisedPDF-{{VERSION}}-x64-portable.zip` | No-install copy for testing: unzip and run `OptimisedPDF.exe` |
| `DefaultAssociations.xml` | Optional: makes Optimised PDF the default PDF app (step 5) |
| `SHA256SUMS.txt` | Checksums of these files |

The installer is **not code-signed**. Intune installs it silently in the system context, which
Defender SmartScreen does not block. Two cases do need attention:
- **App control policies:** if WDAC, App Control for Business or Smart App Control only allows
  signed apps, add a rule for Optimised PDF, by file hash or path.
- **Manual installs:** someone double-clicking the MSI may see "Windows protected your PC".
  Choose More info > Run anyway.

## 1. Add the app

1. In the Intune admin centre, go to **Apps > Windows > Create**.
2. For **App type**, choose **Windows app (Win32)**.
3. Upload `OptimisedPDF-{{VERSION}}-x64.intunewin`. Intune reads the MSI inside it and fills in
   the name, version, product code and default commands. **Change the install command** as in
   step 3.

## 2. App information

| Setting | Value |
|---|---|
| Name | Optimised PDF |
| Publisher | Optimised Accounting Pty Ltd |
| App version | {{VERSION}} |
| Category | Productivity (optional) |

## 3. Program

| Setting | Value |
|---|---|
| Install command | `msiexec /i "{{MSI}}" ALLUSERS=1 /qn` |
| Uninstall command | `msiexec /x {{PRODUCT_CODE}} /qn` |
| Install behaviour | **System** |
| Device restart behaviour | No specific action |
| Return codes | Keep the defaults (0, 1707, 3010, 1641, 1618) |

**`ALLUSERS=1` is required.** Without it the installer defaults to a per-user installation, which
in Intune's system context would install for the SYSTEM account only.

Optional installer properties, added to the install command:

| Property | Default | Effect |
|---|---|---|
| `INSTALLDESKTOPSHORTCUT=0` | 1 | No desktop shortcut. The Start menu shortcut is always created. |
| `INSTALLPDFCONTEXTMENU=0` | 1 | No "Edit with", "Print with" or "Combine files in" Optimised PDF on the right-click menu of PDF files |
| `INSTALLIMAGECONTEXTMENU=0` | 1 | No "Create PDF with Optimised PDF…" on the right-click menu of images |

Example: `msiexec /i "{{MSI}}" ALLUSERS=1 /qn INSTALLDESKTOPSHORTCUT=0`

## 4. Requirements and detection

| Setting | Value |
|---|---|
| Operating system architecture | x64 |
| Minimum operating system | Windows 10 21H2 or later |
| Detection rule | **MSI**, product code `{{PRODUCT_CODE}}`. Intune fills this in. Leave the version check off. |

Assign the app to a device group as **Required**, to install it everywhere, or as **Available for
enrolled devices**, so staff can install it from Company Portal.

## 5. Make it the default PDF app (optional)

Windows doesn't let an installer force itself as a default app. Set it with a policy instead:

1. Go to **Devices > Configuration > Create > Windows 10 and later > Settings catalog**.
2. Search for **Default Associations Configuration** (category *Application Defaults*).
3. Paste the contents of `DefaultAssociations.xml`, encoded as Base64. In PowerShell:
   `[Convert]::ToBase64String([IO.File]::ReadAllBytes("DefaultAssociations.xml"))`
4. Assign it to the same device group. It takes effect at the user's next sign-in.

With Optimised PDF as the default, File Explorer's own **Print** on a PDF uses it too. Either way,
**Print with Optimised PDF** on the right-click menu opens the file, shows the Print dialog and
closes when it is done.

## 6. Updating to a new version

1. Run **Build installer** again. Each release needs a higher version number in `Cargo.toml`.
2. In Intune, add the new `.intunewin` as a new Win32 app, with the same install command.
3. Under **Supersedence**, add the previous version with **Uninstall previous version: No**. The
   MSI upgrades in place: it keeps the same upgrade code and installation folder, and Windows
   Installer replaces the old version.

The app has no update checker of its own. New versions arrive only through Intune.

## What gets installed

- **Program files:** `C:\Program Files\Optimised Accounting\Optimised PDF\`, which holds
  `OptimisedPDF.exe`, `pdfcraft-cli.exe` and the text-recognition models.
- **Start menu and optional desktop shortcut:** "Optimised PDF".
- **PDF handling:** registered as a PDF handler, offered under "Open with" and in Default apps but
  never forced as the default. Optionally adds "Edit with", "Print with" and "Combine files in"
  Optimised PDF on PDF files and "Create PDF with Optimised PDF…" on images.
- **Per-user data:** settings and logs under `%APPDATA%\OptimisedPDF`, recovery files under
  `%LOCALAPPDATA%\OptimisedPDF`. The app starts in dark mode until the user picks another
  appearance. It makes no network connections.

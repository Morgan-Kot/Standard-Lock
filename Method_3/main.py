import webbrowser
import tkinter as tk
from ciphers import CIPHERS


class FormulaDialog(tk.Toplevel):
    def __init__(self, parent, cipher_cls):
        super().__init__(parent)
        self.title(f"Formula Info: {cipher_cls.name}")
        self.geometry("440x260")
        self.resizable(False, False)

        BG_COLOR = "#d4d0c8"
        TEXT_COLOR = "#000000"
        self.configure(bg=BG_COLOR)

        self.transient(parent)
        self.grab_set()

        # Header Title
        lbl_title = tk.Label(
            self,
            text=cipher_cls.name,
            font=("MS Sans Serif", 10, "bold"),
            bg=BG_COLOR,
            fg=TEXT_COLOR,
        )
        lbl_title.pack(anchor="w", padx=12, pady=(10, 4))

        # Explanation Box Frame
        exp_frame = tk.LabelFrame(
            self,
            text=" Cipher Explanation ",
            bg=BG_COLOR,
            fg=TEXT_COLOR,
            font=("MS Sans Serif", 8, "bold"),
            bd=2,
            relief="groove",
        )
        exp_frame.pack(fill="both", expand=True, padx=10, pady=4)

        lbl_desc = tk.Label(
            exp_frame,
            text=cipher_cls.desc,
            font=("MS Sans Serif", 9),
            bg="#ffffff",
            fg="#000000",
            anchor="nw",
            justify="left",
            wraplength=390,
            bd=2,
            relief="sunken",
        )
        lbl_desc.pack(fill="both", expand=True, padx=6, pady=6)

        # Clickable GitHub Link
        repo_url = getattr(
            cipher_cls,
            "github_url",
            "https://github.com/your-username/encryption-center",
        )
        lbl_link = tk.Label(
            self,
            text="View Documentation / GitHub Repo",
            font=("MS Sans Serif", 8, "underline"),
            bg=BG_COLOR,
            fg="#000000" if parent.tk.call("tk", "windowingsystem") == "win32" else "blue",
            cursor="hand2",
        )
        lbl_link.pack(pady=(2, 0))
        lbl_link.bind("<Button-1>", lambda e: webbrowser.open_new(repo_url))

        # OK Button
        btn_close = tk.Button(
            self,
            text="OK",
            width=10,
            command=self.destroy,
            bg=BG_COLOR,
            fg=TEXT_COLOR,
            relief="raised",
            bd=2,
        )
        btn_close.pack(pady=8)


class EncryptionCenter(tk.Tk):
    def __init__(self):
        super().__init__()
        self.title("Encrypter++")
        self.geometry("480x800")

        BG_COLOR = "#d4d0c8"
        TEXT_COLOR = "#000000"
        self.configure(bg=BG_COLOR)

        self.updating = False
        self.selected_cipher_name = tk.StringVar(value=list(CIPHERS.keys())[0])

        # --- Menu Bar ---
        menubar = tk.Menu(self, bg=BG_COLOR, fg=TEXT_COLOR)
        file_menu = tk.Menu(menubar, tearoff=0, bg=BG_COLOR, fg=TEXT_COLOR)
        file_menu.add_command(label="Clear All", command=self.clear_fields)
        file_menu.add_separator()
        file_menu.add_command(label="Exit", command=self.quit)
        menubar.add_cascade(label="File", menu=file_menu)

        formulas_menu = tk.Menu(menubar, tearoff=0, bg=BG_COLOR, fg=TEXT_COLOR)
        formulas_menu.add_command(
            label="View Active Cipher Info", command=self.show_formula_info
        )
        menubar.add_cascade(label="Formulas", menu=formulas_menu)

        self.config(menu=menubar)

        # --- Top Header & Algorithm Selection ---
        top_frame = tk.Frame(self, bg=BG_COLOR, bd=2, relief="groove")
        top_frame.pack(fill="x", padx=8, pady=6)

        title_lbl = tk.Label(
            top_frame,
            text="Encryption Center",
            font=("MS Sans Serif", 10, "bold"),
            bg=BG_COLOR,
            fg=TEXT_COLOR,
        )
        title_lbl.pack(side="left", padx=5, pady=5)

        cipher_dropdown = tk.OptionMenu(
            top_frame,
            self.selected_cipher_name,
            *CIPHERS.keys(),
            command=self.on_cipher_change,
        )
        cipher_dropdown.config(
            bg=BG_COLOR,
            fg=TEXT_COLOR,
            activebackground="#0a246a",
            activeforeground="#ffffff",
            highlightthickness=1,
            relief="raised",
        )
        cipher_dropdown.pack(side="right", padx=5, pady=3)

        lbl_select = tk.Label(
            top_frame,
            text="Method:",
            font=("MS Sans Serif", 8),
            bg=BG_COLOR,
            fg=TEXT_COLOR,
        )
        lbl_select.pack(side="right", padx=2)

        # --- Input Areas ---
        main_frame = tk.Frame(self, bg=BG_COLOR)
        main_frame.pack(fill="both", expand=True, padx=8, pady=4)

        plain_frame = tk.LabelFrame(
            main_frame,
            text=" Plaintext / Input ",
            bg=BG_COLOR,
            fg=TEXT_COLOR,
            font=("MS Sans Serif", 8, "bold"),
            relief="groove",
            bd=2,
        )
        plain_frame.pack(fill="both", expand=True, pady=4)

        self.txt_plain = tk.Text(
            plain_frame,
            wrap="word",
            bg="#ffffff",
            fg="#000000",
            bd=2,
            relief="sunken",
            font=("Courier", 9),
        )
        self.txt_plain.pack(fill="both", expand=True, padx=4, pady=4)

        cipher_frame = tk.LabelFrame(
            main_frame,
            text=" Ciphertext / Encrypted Output ",
            bg=BG_COLOR,
            fg=TEXT_COLOR,
            font=("MS Sans Serif", 8, "bold"),
            relief="groove",
            bd=2,
        )
        cipher_frame.pack(fill="both", expand=True, pady=4)

        self.txt_cipher = tk.Text(
            cipher_frame,
            wrap="word",
            bg="#ffffff",
            fg="#000000",
            bd=2,
            relief="sunken",
            font=("Courier", 9),
        )
        self.txt_cipher.pack(fill="both", expand=True, padx=4, pady=4)

        self.txt_plain.bind("<KeyRelease>", self.on_plain_type)
        self.txt_cipher.bind("<KeyRelease>", self.on_cipher_type)

    def get_cipher_class(self):
        return CIPHERS[self.selected_cipher_name.get()]

    def on_plain_type(self, event=None):
        if self.updating:
            return
        self.updating = True
        try:
            cipher_cls = self.get_cipher_class()
            plain_text = self.txt_plain.get("1.0", tk.END).rstrip("\n")
            encrypted = cipher_cls.encrypt(plain_text)

            self.txt_cipher.delete("1.0", tk.END)
            self.txt_cipher.insert("1.0", encrypted)
        except Exception:
            pass
        finally:
            self.updating = False

    def on_cipher_type(self, event=None):
        if self.updating:
            return
        self.updating = True
        try:
            cipher_cls = self.get_cipher_class()
            cipher_text = self.txt_cipher.get("1.0", tk.END).rstrip("\n")
            decrypted = cipher_cls.decrypt(cipher_text)

            self.txt_plain.delete("1.0", tk.END)
            self.txt_plain.insert("1.0", decrypted)
        except Exception:
            pass
        finally:
            self.updating = False

    def on_cipher_change(self, value):
        self.on_plain_type()

    def clear_fields(self):
        self.txt_plain.delete("1.0", tk.END)
        self.txt_cipher.delete("1.0", tk.END)

    def show_formula_info(self):
        cipher_cls = self.get_cipher_class()
        FormulaDialog(self, cipher_cls)


if __name__ == "__main__":
    app = EncryptionCenter()
    app.mainloop()
class Toast {
    private readonly _content: string;
    private readonly _containerElement: HTMLElement;
    private readonly _toastElement: HTMLElement;

    constructor(content: string, containerElement: HTMLElement) {
        this._content = content;

        this._containerElement = containerElement;
        this._toastElement = this.createToastElement();
    }

    show() {
        this._containerElement.appendChild(this._toastElement);

        new bootstrap.Toast(this._toastElement);

        setTimeout(() => this._toastElement.classList.add('show'), 50);
        setTimeout(() => this.hide(), 4000);
        setTimeout(() => this._containerElement.removeChild(this._toastElement), 5000);
    }

    hide() {
        this._toastElement.classList.remove('show');
    }

    private createToastElement(): HTMLElement {
        const html =
            '<div class="toast align-items-center" role="alert" aria-live="assertive" aria-atomic="true">'+
            '    <div class="d-flex">'+
            '        <div class="toast-body">'+
            '            '+this._content+
            '        </div>'+
            '        <button type="button" class="btn-close me-2 m-auto" data-bs-dismiss="toast" aria-label="Close"></button>'+
            '    </div>'+
            '</div>'
        ;

        const div = document.createElement('div');

        div.innerHTML = html;

        const toast = div.firstChild;

        if (!(toast instanceof HTMLElement)) {
            throw 'Toast element could not be created successfully.';
        }

        return toast;
    }
}

function createContainerElement(): HTMLElement {
    const html = '<div class="toast-container position-absolute p-3 top-0 start-0"></div>';

    const div = document.createElement('div');

    div.innerHTML = html;

    const container = div.firstChild;

    if (!(container instanceof HTMLElement)) {
        throw 'Toast container could not be created successfully.';
    }

    return container;
}

const container = createContainerElement();
let addedToBody = false;

function message(content: string) {
    const toast = new Toast(content, container);

    if (!addedToBody) {
        document.body.appendChild(container);
        addedToBody = true;
    }

    toast.show();
}

export default message;
